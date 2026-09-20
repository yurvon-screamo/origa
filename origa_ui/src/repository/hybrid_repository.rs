use std::sync::Arc;

use futures::lock::Mutex;
use ulid::Ulid;

use origa::{
    domain::{OrigaError, User},
    traits::UserRepository,
    use_cases::{PROBE_SKIP_FULL_CHECK_INTERVAL, SyncMeta},
};

use crate::repository::file_repository::FileSystemUserRepository;
use crate::repository::sync_meta_store::{IdbSyncMetaStore, SyncMetaStore};
use crate::repository::trailbase_repository::{
    RemoteRow, RemoteUserSource, TrailBaseUserRepository,
};

#[cfg(test)]
#[path = "hybrid_sync_tests.rs"]
mod sync_tests;

/// Shared in-flight gate serializing `merge_current_user` passes. Clones of
/// the repository share one gate, so the lesson-complete screen's sync and
/// the home mount's sync never run as two concurrent full merges (each a
/// multi-MB GET + PATCH + GET competing for the same uplink) — the second
/// caller waits, then runs its own pass, which takes the cheap skip path
/// when nothing changed since.
pub(crate) type SyncGate = Arc<Mutex<()>>;

#[derive(Clone)]
pub struct HybridUserRepository {
    local: FileSystemUserRepository,
    remote: TrailBaseUserRepository,
    meta: IdbSyncMetaStore,
    sync_gate: SyncGate,
}

impl HybridUserRepository {
    pub fn new() -> Self {
        Self {
            local: FileSystemUserRepository::new(),
            remote: TrailBaseUserRepository::new(),
            meta: IdbSyncMetaStore,
            sync_gate: Arc::new(Mutex::new(())),
        }
    }

    pub async fn merge_current_user(&self) -> Result<(), OrigaError> {
        sync_merge_gated(&self.local, &self.remote, &self.meta, &self.sync_gate).await
    }

    /// Definitive remote-miss probe: does the server hold a user record for
    /// the signed-in session RIGHT NOW? Used by the login profile bootstrap
    /// to discriminate "genuinely first login" from "the merge returned
    /// without seeding the local store while the remote record is alive" —
    /// minting a fresh empty profile in the latter case shadows the
    /// canonical record on every device that logs in afterwards (#492).
    pub async fn has_remote_record(&self) -> Result<bool, OrigaError> {
        Ok(self.remote.find_current_raw().await?.is_some())
    }

    /// Marks the local user record as mutated: the next sync must take the
    /// full merge path. Called after every user-action write (`save`,
    /// `save_sync`) — the store write is a tiny IndexedDB record, accepted
    /// per ADR-045 (future work: co-locate it with the user write in one
    /// transaction).
    async fn mark_local_dirty(&self) {
        if let Err(e) = mark_dirty(&self.meta).await {
            tracing::warn!("Failed to persist sync dirty flag: {e:?}");
        }
    }

    /// Delete the remote user record only. Unlike `delete`, this does NOT
    /// swallow remote errors — account deletion must surface failures so the
    /// caller (AuthStore) can abort the flow instead of leaving the user in a
    /// half-deleted state. Local data cleanup is the caller's responsibility.
    ///
    /// `user_id` is accepted as `Option<Ulid>` — when `None`, the caller has no
    /// loaded domain `User`, which means the account is in an anomalous state
    /// (authenticated session but no User object). This is surfaced as an error
    /// rather than silently passing a nil ULID.
    pub async fn delete_remote(&self, user_id: Option<Ulid>) -> Result<(), OrigaError> {
        tracing::info!("delete_remote: Deleting remote user {:?}", user_id);
        let id = user_id.ok_or_else(|| OrigaError::RepositoryError {
            reason: "Cannot delete account: no user is currently loaded".to_string(),
        })?;
        // The sync meta belongs to the deleted account: reset it so a future
        // login cannot inherit a stale skip fingerprint.
        if let Err(e) = self.meta.store(&SyncMeta::unsynced()).await {
            tracing::warn!("Failed to reset sync meta after account deletion: {e:?}");
        }
        self.remote.delete(id).await
    }
}

/// Loads, dirties and persists the meta in one step. Returns the stored
/// state so the caller can capture `dirty_epoch` **after** its own
/// `mark_dirty` for the CAS check in `record_sync` (ADR-045).
async fn mark_dirty(meta_store: &impl SyncMetaStore) -> Result<SyncMeta, OrigaError> {
    let mut meta = meta_store.load().await?;
    meta.mark_dirty();
    meta_store.store(&meta).await?;
    Ok(meta)
}

/// Cheap existence check for the local user record, used by the sync
/// skip-path: the short-circuit must not fire when the local store is
/// missing or corrupted, because the full path is what re-seeds it
/// (ADR-045). Implemented via an IndexedDB key count — no user parsing.
pub(crate) trait LocalUserPresence {
    fn has_any_user(&self) -> impl Future<Output = Result<bool, OrigaError>>;
}

/// The explicit-checkpoint push core (ADR-045), generic so it runs
/// against in-memory spies in native tests: mark dirty (crash safety),
/// write local, then push remote.
pub(crate) async fn save_sync_core<F, Fut>(
    local: &impl UserRepository,
    meta_store: &impl SyncMetaStore,
    user: &User,
    push: F,
) -> Result<(), OrigaError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<(), OrigaError>>,
{
    save_sync_core_with_delay(
        crate::repository::sync_retry::SYNC_RETRY_DELAY_MS as u32,
        local,
        meta_store,
        user,
        push,
    )
    .await
}

/// [`save_sync_core`] with the retry delay injectable for tests (the
/// native harness blocks the thread on the backoff sleep).
async fn save_sync_core_with_delay<F, Fut>(
    delay_ms: u32,
    local: &impl UserRepository,
    meta_store: &impl SyncMetaStore,
    user: &User,
    push: F,
) -> Result<(), OrigaError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<(), OrigaError>>,
{
    // Dirty BEFORE the local write, and it stays set across the whole
    // remote push: the push takes seconds for a large knowledge set
    // (serialization + deflate + upload) — exactly the jetsam window
    // this ADR exists for — and a crash there must not leave a clean
    // meta that skips the next sync (ADR-045).
    if let Err(e) = mark_dirty(meta_store).await {
        tracing::warn!("Failed to persist sync dirty flag: {e:?}");
    }
    local.save(user).await?;
    // The single shared sync retry (see `should_retry_sync_error`): one
    // transient edge flap must not fail the checkpoint. Idempotent by
    // construction — the push re-resolves the row before creating, so a
    // retried push PATCHes the row the first attempt landed instead of
    // duplicating it.
    crate::repository::sync_retry::with_sync_retry_delay(delay_ms, push).await
}

/// The sync orchestration core (ADR-045), generic over the repositories so
/// it runs against in-memory spies in native tests.
///
/// Steady state (nothing changed since the last successful sync) costs one
/// raw remote fetch plus a fingerprint comparison — the multi-megabyte
/// inflate/parse/serialize cycle of a large knowledge set never runs. Any
/// difference takes the full path: decode the remote row, merge into the
/// local user, push, then record the **server-authoritative** fingerprint
/// re-fetched after the push.
pub(crate) async fn sync_merge(
    local: &(impl UserRepository + LocalUserPresence),
    remote: &impl RemoteUserSource,
    meta_store: &impl SyncMetaStore,
) -> Result<(), OrigaError> {
    // Local bookkeeping first (ordering note: a broken local meta-store now
    // surfaces before any network error in the logs).
    let meta = meta_store.load().await?;
    let local_exists = local.has_any_user().await?;

    // Delta probe (ADR-045 Future work): when the state is clean and the
    // probe fields are known, ask the server about the exact row instead
    // of downloading it. Fail-open on any probe error — behaviour falls
    // back to today's full fetch. The target is materialized owned: the
    // probe answer travels across awaits and the meta must stay free for
    // the skip-counter write below.
    let probe_target = meta
        .probe_target()
        .map(|(record_id, stamp)| (record_id, stamp.to_string()));
    let mut probed_row: Option<RemoteRow> = None;
    if let Some((record_id, seen_updated_at)) = probe_target {
        if local_exists {
            match remote.fetch_if_changed(record_id, &seen_updated_at).await {
                Ok(None) => {
                    // Unchanged. RELOAD the meta first: a parallel `save()`
                    // (which does not take the sync gate) may have dirtied
                    // it while the probe was in flight — the pre-await
                    // snapshot must never be written back, or the dirty
                    // flag is silently lost (ADR-045's CAS discipline).
                    let fresh = meta_store.load().await?;
                    if fresh.dirty {
                        tracing::debug!("Local save landed during the probe; taking the full path");
                        // Fall through: the dirty local must merge and push
                        // regardless of the remote verdict.
                    } else if fresh.probe_skips_since_full < PROBE_SKIP_FULL_CHECK_INTERVAL {
                        // The periodic valve: after enough consecutive
                        // skips one full check runs anyway — the bounded
                        // answer to every false-negative class the probe
                        // can have (clock skew, timestamp collisions,
                        // format drift, server-side row deletion).
                        let mut counted = fresh;
                        counted.probe_skips_since_full += 1;
                        meta_store.store(&counted).await?;
                        tracing::debug!(
                            skips = counted.probe_skips_since_full,
                            "Sync skipped by delta probe: row unchanged"
                        );
                        return Ok(());
                    } else {
                        tracing::debug!(
                            skips = fresh.probe_skips_since_full,
                            "Delta-probe valve: running the full check"
                        );
                        // Fall through to the full path.
                    }
                },
                Ok(Some(row)) => {
                    // The row itself — either genuinely changed or a false
                    // alarm (stamp bumped, content identical); the
                    // fingerprint comparison below decides, and the skip
                    // branch heals the stamp either way.
                    probed_row = Some(row);
                },
                Err(e) => {
                    tracing::debug!(error = %e, "Delta probe failed; falling back to full fetch");
                },
            }
        }
    }

    // The create/restore branches are reachable only through this fetch:
    // the probe answers empty for both "unchanged" and "row gone" and
    // never observes a missing row.
    let raw = match probed_row {
        Some(row) => Some(row),
        None => remote.find_current_raw().await?,
    };

    let Some(remote_row) = raw else {
        // No remote record: seed the server from local (fresh install or a
        // remote wiped elsewhere).
        match local.get_current_user().await? {
            Some(local_user) => {
                tracing::info!("Creating remote user from local");
                remote.create(&local_user).await?;
                record_post_create_fingerprint(remote, meta_store).await?;
            },
            None => tracing::warn!("No user found locally or remotely"),
        }
        return Ok(());
    };

    let record_id = remote_row.record_id;
    // Captured before `into_user` consumes the row: the restore branch
    // below records the fetched fingerprint without a redundant push, and
    // every branch records the probe bookkeeping from these two values.
    let remote_fingerprint = remote_row.fingerprint.clone();
    let remote_updated_at = remote_row.updated_at_rfc3339();

    // The skip requires a present local record: when the local store has
    // no user key, the merge below is the recovery path that re-seeds it
    // from the remote row, and skipping it would strand the user with a
    // clean fingerprint and no local data (ADR-045). The probe is a keyed
    // count — a corrupted-but-present record is NOT detected here (see the
    // ADR threat model for the residual risk).
    //
    // The decision runs on a RE-LOADED meta: a parallel save may have
    // dirtied it since the pre-fetch snapshot, and a dirty local must
    // always take the merge path regardless of the remote verdict.
    let current = meta_store.load().await?;
    if !current.should_skip(&remote_fingerprint) || !local_exists {
        // Fall through to the merge path (restore or full). Both re-load
        // the meta internally via `mark_dirty`, so the stale outer
        // snapshot is not consulted again.
    } else {
        tracing::debug!("Sync skipped: remote unchanged since last sync");
        // Heal the probe bookkeeping from the row already in hand,
        // applying ONLY the probe delta to the fresh snapshot: a false
        // probe alarm (stamp bumped, content identical) settles here, so
        // the next trigger is a single cheap probe again. `record_probe_row`
        // touches neither the dirty flag nor the epoch.
        let mut healed = current;
        healed.record_probe_row(record_id, remote_updated_at);
        meta_store.store(&healed).await?;
        return Ok(());
    }

    let remote_user = remote_row.into_user()?;

    match local.get_current_user().await? {
        None => {
            tracing::info!("Restoring local user from remote");
            // The local content is seeded from the fetched row, so pushing
            // it back would ship megabytes of identical data. Write the
            // local record and record the fetched fingerprint directly;
            // concurrent server writes surface as a fingerprint change on
            // the next sync.
            //
            // Ordering (#492): the small sync-meta write goes FIRST, the
            // multi-MB user row SECOND. Both records share the `users`
            // object store, and IndexedDB serializes write transactions on
            // a store — a small write queued AFTER a huge structural-clone
            // put was observed to starve indefinitely (the merge never
            // resolved, the restore window never closed). Crash-safety is
            // unchanged: if the process dies between the two writes, the
            // recorded fingerprint is already settled but `local_exists`
            // is still false, and the skip-path's local-existence conjunct
            // routes the next sync back into this restore branch (ADR-045).
            let mut meta = mark_dirty(meta_store).await?;
            let observed_epoch = meta.dirty_epoch;
            tracing::info!("Restore: settling sync meta before the heavy user write");
            meta.record_sync(remote_fingerprint, observed_epoch);
            meta.record_probe_row(record_id, remote_updated_at);
            meta_store.store(&meta).await?;
            tracing::info!("Restore: sync meta stored, saving local user…");
            local.save(&remote_user).await?;
            tracing::info!("Restore: complete");
            Ok(())
        },
        Some(mut local_user) => {
            tracing::info!("Merging remote into local user");
            local_user.merge(&remote_user);
            full_sync_cycle(local, remote, meta_store, local_user, record_id).await
        },
    }
}

/// [`sync_merge`] under the shared in-flight gate. Concurrent callers
/// (lesson-complete sync vs home mount sync, auth bootstrap vs home mount)
/// serialize: the loser waits for the winner's pass, then runs its own —
/// a clean meta plus matching fingerprint collapses it to the cheap skip
/// path, a fresh local save (dirty flag) still forces the full path.
pub(crate) async fn sync_merge_gated(
    local: &(impl UserRepository + LocalUserPresence),
    remote: &impl RemoteUserSource,
    meta_store: &impl SyncMetaStore,
    gate: &SyncGate,
) -> Result<(), OrigaError> {
    let _permit = gate.lock().await;
    // RESIDUAL RACE (pre-existing ADR-045 limitation, documented — not
    // fixed here): a local save whose `mark_dirty` lands inside the window
    // between this sync's `local.get_current_user()` and its own
    // `mark_dirty` loses the CAS epoch — the sync pushes a snapshot taken
    // before that save, overwrites the local record with it, and clears
    // the dirty flag, deferring the save to the next unrelated mutation.
    // The gate narrows this window (no concurrent sync can interleave its
    // meta writes) but cannot close it for user-action saves.
    sync_merge(local, remote, meta_store).await
}

/// The full path: mark dirty (crash-safety), write local, push remote, then
/// record the server-authoritative fingerprint with the epoch captured
/// after the own `mark_dirty` (concurrent mutations keep the flag set).
async fn full_sync_cycle(
    local: &impl UserRepository,
    remote: &impl RemoteUserSource,
    meta_store: &impl SyncMetaStore,
    user: User,
    record_id: i64,
) -> Result<(), OrigaError> {
    // Dirty BEFORE the local write: a crash between the local save and the
    // remote push must leave the flag set so the next sync re-pushes.
    let meta = mark_dirty(meta_store).await?;
    let observed_epoch = meta.dirty_epoch;

    local.save(&user).await?;
    remote.save_with_record_id(record_id, &user).await?;

    record_sync_fingerprint(remote, meta_store, observed_epoch).await
}

/// Re-fetches the raw row and records its fingerprint. The fingerprint is
/// server-authoritative on purpose: deriving it from the request body would
/// silently break skip matching whenever the server normalizes anything on
/// storage.
async fn record_sync_fingerprint(
    remote: &impl RemoteUserSource,
    meta_store: &impl SyncMetaStore,
    observed_epoch: u64,
) -> Result<(), OrigaError> {
    let mut meta = meta_store.load().await?;
    match remote.find_current_raw().await? {
        Some(fresh) => {
            let fingerprint = fresh.fingerprint.clone();
            let record_id = fresh.record_id;
            let updated_at = fresh.updated_at_rfc3339();
            meta.record_sync(fingerprint, observed_epoch);
            meta.record_probe_row(record_id, updated_at);
            meta_store.store(&meta).await?;
        },
        None => tracing::warn!("Remote row vanished after push; sync meta left dirty"),
    }
    Ok(())
}

/// Post-create variant: identical epoch semantics, but no local write —
/// the local user already exists by construction.
async fn record_post_create_fingerprint(
    remote: &impl RemoteUserSource,
    meta_store: &impl SyncMetaStore,
) -> Result<(), OrigaError> {
    let meta = mark_dirty(meta_store).await?;
    record_sync_fingerprint(remote, meta_store, meta.dirty_epoch).await
}

impl UserRepository for HybridUserRepository {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        self.local.get_current_user().await
    }

    // Local-only write on the hot path. Rating a card, marking it known, or
    // creating one are high-frequency actions; awaiting a remote round-trip
    // here would block the core study loop (especially on mobile). The local
    // write is authoritative for the device; cross-device propagation happens
    // through `save_sync` at explicit checkpoints (onboarding, imports, auth)
    // and through `merge_current_user` on login. The user id is already
    // canonical thanks to the session-derived ULID, so a local-only save is
    // correctly attributed to the right identity.
    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        tracing::info!("save: Starting local save for user {}", user.id());
        // Dirty BEFORE the local write — the same crash-safety invariant as
        // the full sync cycle: a crash between the write and the flag must
        // leave the flag set, otherwise the next sync silently skips and
        // the written data is never pushed (ADR-045).
        self.mark_local_dirty().await;
        self.local.save(user).await?;
        tracing::info!("save: Local save completed for user {}", user.id());
        Ok(())
    }

    // Explicit sync checkpoint: local + remote. Used by auth, onboarding, and
    // imports where a network round-trip is acceptable and the data must reach
    // the server before the user can switch devices.
    //
    // The local write runs first so the device stays usable offline even when
    // the network is down. Remote failures are then surfaced as `Err` instead
    // of being swallowed: a silent `Ok` here is what allowed the cross-device
    // split-progress bug, because the initial profile create would log a remote
    // error and return `Ok`, so the user moved on without a canonical remote
    // record and the next device's login found nothing to merge against.
    //
    // The remote push goes through the single sync retry (`save_sync_core`):
    // a bare push here is what let one transient proxy flap kill a whole
    // Yandex registration (2026-09-20) while the manual retry minutes later
    // succeeded. Surfacing after the retry still holds.
    async fn save_sync(&self, user: &User) -> Result<(), OrigaError> {
        tracing::info!("save_sync: Starting save for user {}", user.id());
        // The push closure must own its data (FnMut -> owned futures), and
        // `User` can carry a multi-MB knowledge set — so ONE deep clone
        // behind an Arc, and every attempt only bumps the refcount. The
        // repository itself is cheap to clone (String + client handle).
        let remote = self.remote.clone();
        let pushed = Arc::new(user.clone());
        let result = save_sync_core(&self.local, &self.meta, user, move || {
            let remote = remote.clone();
            let pushed = Arc::clone(&pushed);
            async move { remote.save(&pushed).await }
        })
        .await;

        match result {
            Ok(()) => {
                tracing::info!("save_sync: Remote save completed for user {}", user.id());
                // The push succeeded but its fingerprint is not recorded here: the
                // next `merge_current_user` takes the full path (dirty) and records
                // the server-authoritative fingerprint — one fewer raw fetch per
                // checkpoint than recording inline (ADR-045).
                Ok(())
            },
            Err(e) => {
                tracing::error!(
                    "save_sync: Remote save failed for user {}: {:?}. Local save kept; surfacing error to caller.",
                    user.id(),
                    e
                );
                Err(e)
            },
        }
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        tracing::info!("delete: Deleting user {}", user_id);

        // Always delete local data first
        if let Err(e) = self.local.delete(user_id).await {
            tracing::error!("delete: Local delete failed: {:?}", e);
            return Err(e);
        }
        tracing::info!("delete: Local delete completed for user {}", user_id);

        // Try remote delete, but don't fail if it doesn't work
        match self.remote.delete(user_id).await {
            Ok(_) => tracing::info!("delete: Remote delete completed for user {}", user_id),
            Err(e) => {
                tracing::error!(
                    "delete: Remote delete failed for user {}: {:?}. Local data deleted.",
                    user_id,
                    e
                );
            },
        }

        Ok(())
    }
}
