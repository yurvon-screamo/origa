//! Native tests for the sync orchestration core (`sync_merge`, ADR-045).
//!
//! The spies mirror the contracts of the real repositories without any
//! JavaScript: `SpyLocal` implements `UserRepository`, `SpyRemote`
//! implements `RemoteUserSource` while simulating a server that stores the
//! pushed row verbatim (and may "normalize" it — e.g. add columns — to
//! exercise the post-push reconstruction divergence paths of ADR-060).

use std::future::Future;
use std::sync::{Arc, Mutex};

use futures::future::poll_fn;
use origa::domain::{OrigaError, User};
use origa::traits::UserRepository;
use origa::use_cases::PROBE_SKIP_FULL_CHECK_INTERVAL;
use serde_json::{Value, json};
use ulid::Ulid;

use super::{SyncGate, save_sync_core_with_delay, sync_merge, sync_merge_gated};
use crate::repository::hybrid_repository::LocalUserPresence;
use crate::repository::sync_meta_store::{InMemorySyncMetaStore, SyncMetaStore};
use crate::repository::trailbase_repository::{
    RemoteRow, RemoteUserSource, SavedWireRow, remote_row_from_value, user_to_json,
};

/// Alias kept short for the fixtures below.
type MetaStore = InMemorySyncMetaStore;

// ═══════════════════════════════════════════════════════════════════════
// Spies
// ═══════════════════════════════════════════════════════════════════════

#[derive(Clone, Default)]
struct SpyLocal {
    state: Arc<Mutex<Option<User>>>,
    saves: Arc<Mutex<Vec<Ulid>>>,
    fail_saves: bool,
}

impl SpyLocal {
    fn with_user(user: User) -> Self {
        Self {
            state: Arc::new(Mutex::new(Some(user))),
            saves: Arc::new(Mutex::new(Vec::new())),
            fail_saves: false,
        }
    }

    /// Every `save` fails — simulates a broken local store.
    fn with_failing_saves(mut self) -> Self {
        self.fail_saves = true;
        self
    }

    fn save_count(&self) -> usize {
        self.saves.lock().unwrap().len()
    }
}

impl UserRepository for SpyLocal {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        Ok(self.state.lock().unwrap().clone())
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        if self.fail_saves {
            return Err(OrigaError::RepositoryError {
                reason: "simulated local save failure".to_string(),
            });
        }
        self.saves.lock().unwrap().push(user.id());
        *self.state.lock().unwrap() = Some(user.clone());
        Ok(())
    }

    async fn delete(&self, _user_id: Ulid) -> Result<(), OrigaError> {
        *self.state.lock().unwrap() = None;
        Ok(())
    }
}

impl LocalUserPresence for SpyLocal {
    async fn has_any_user(&self) -> Result<bool, OrigaError> {
        Ok(self.state.lock().unwrap().is_some())
    }
}

/// Simulated TrailBase row store. `normalize_on_save` mimics server-side
/// normalization (e.g. a column the client did not send) to prove the
/// recorded fingerprint comes from the re-fetched server bytes.
struct SpyRemote {
    rows: Arc<Mutex<Vec<Value>>>,
    fetches: Arc<Mutex<usize>>,
    pushes: Arc<Mutex<Vec<i64>>>,
    creates: Arc<Mutex<Vec<Ulid>>>,
    probes: Arc<Mutex<usize>>,
    /// Programmed delta-probe verdict for the NEXT probe call; `None`
    /// derives the answer from the stored rows (id match + `$gt` on
    /// `updated_at`, mirroring the server-side filter).
    probe_answer: Mutex<Option<Result<bool, OrigaError>>>,
    /// Hook executed right before a probe answers — simulates a parallel
    /// user action (card rated → `save()` → `mark_dirty`) landing inside
    /// the probe's in-flight window.
    on_probe: Option<Box<dyn Fn() + Send + Sync>>,
    normalize_on_save: bool,
    fail_pushes: bool,
    /// Hook executed right after a push lands — used to simulate a
    /// concurrent user action (card rated) inside the sync window.
    on_push: Option<Box<dyn Fn() + Send + Sync>>,
}

impl SpyRemote {
    fn new(rows: Vec<Value>) -> Self {
        Self {
            rows: Arc::new(Mutex::new(rows)),
            fetches: Arc::new(Mutex::new(0)),
            pushes: Arc::new(Mutex::new(Vec::new())),
            creates: Arc::new(Mutex::new(Vec::new())),
            probes: Arc::new(Mutex::new(0)),
            probe_answer: Mutex::new(None),
            normalize_on_save: false,
            fail_pushes: false,
            on_push: None,
            on_probe: None,
        }
    }

    /// The row a probe for `record_id` with `seen_updated_at` would answer,
    /// derived from the stored rows exactly like the server-side filter:
    /// the row with that id whose `updated_at` sorts past the stamp.
    fn probe_row(&self, record_id: i64, seen_updated_at: &str) -> Option<Value> {
        let rows = self.rows.lock().unwrap();
        rows.iter()
            .find(|row| row.get("id").and_then(Value::as_i64) == Some(record_id))
            .filter(|row| {
                row.get("updated_at")
                    .and_then(Value::as_str)
                    .is_some_and(|ts| ts > seen_updated_at)
            })
            .cloned()
    }

    fn row_for_fetch(&self) -> Result<Option<RemoteRow>, OrigaError> {
        *self.fetches.lock().unwrap() += 1;
        let rows = self.rows.lock().unwrap();
        let selected = rows
            .iter()
            .filter_map(|row| {
                let id = row.get("id").and_then(Value::as_i64)?;
                Some((id, row))
            })
            .min_by_key(|(id, _)| *id)
            .map(|(_, row)| row.clone());
        match selected {
            Some(row) => remote_row_from_value(row).map(Some),
            None => Ok(None),
        }
    }

    fn store_pushed_user(&self, record_id: i64, user: &User) -> Result<SavedWireRow, OrigaError> {
        let trailbase_id = TRAILBASE_ID
            .with(|id| id.borrow().clone())
            .unwrap_or_else(|| "00000000-0000-0000-0000-000000000001".to_string());
        // The pushed payload — exactly what the client sent. The real
        // repository's wire carries no id (it travels in the URL) and no
        // server-side columns; the reconstruction must derive from THIS
        // shape, not from the stored row.
        let pushed = user_to_json(user, &trailbase_id)?;

        // The stored server row: pushed payload + id + simulated
        // server-side normalization (columns the client never sends).
        let mut row = pushed.clone();
        row["id"] = json!(record_id);
        if self.normalize_on_save {
            row["server_generated_field"] = json!("normalized");
        }

        let mut rows = self.rows.lock().unwrap();
        rows.retain(|r| r.get("id").and_then(Value::as_i64) != Some(record_id));
        rows.push(row);
        Ok(SavedWireRow {
            record_id,
            wire: pushed,
        })
    }
}

impl RemoteUserSource for SpyRemote {
    async fn find_current_raw(&self) -> Result<Option<RemoteRow>, OrigaError> {
        self.row_for_fetch()
    }

    async fn save_with_record_id(
        &self,
        record_id: i64,
        user: &User,
    ) -> Result<SavedWireRow, OrigaError> {
        if self.fail_pushes {
            return Err(OrigaError::RepositoryError {
                reason: "simulated push failure".to_string(),
            });
        }
        let saved = self.store_pushed_user(record_id, user)?;
        self.pushes.lock().unwrap().push(record_id);
        if let Some(hook) = &self.on_push {
            hook();
        }
        Ok(saved)
    }

    async fn create(&self, user: &User) -> Result<SavedWireRow, OrigaError> {
        let record_id = 42;
        let saved = self.store_pushed_user(record_id, user)?;
        self.creates.lock().unwrap().push(user.id());
        Ok(saved)
    }

    async fn fetch_if_changed(
        &self,
        record_id: i64,
        seen_updated_at: &str,
    ) -> Result<Option<RemoteRow>, OrigaError> {
        *self.probes.lock().unwrap() += 1;
        if let Some(hook) = &self.on_probe {
            hook();
        }
        let verdict = self
            .probe_answer
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| Ok(self.probe_row(record_id, seen_updated_at).is_some()));
        match verdict {
            Err(e) => Err(e),
            Ok(false) => Ok(None),
            Ok(true) => {
                let row = self.probe_row(record_id, seen_updated_at).ok_or_else(|| {
                    OrigaError::RepositoryError {
                        reason: "programmed probe-fire but no matching row".to_string(),
                    }
                })?;
                remote_row_from_value(row).map(Some)
            },
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Fixtures
// ═══════════════════════════════════════════════════════════════════════

thread_local! {
    static TRAILBASE_ID: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn fixture_user(email: &str) -> User {
    User::new(
        email.to_string(),
        origa::domain::NativeLanguage::Russian,
        None,
    )
}

fn fixture_row(email: &str) -> Value {
    json!({
        "id": 7,
        "trailbase_id": "018f3f21-7f9f-7bbb-ade0-8d4d9e16c7e1",
        "username": "fixture",
        "email": email,
        "native_language": 0,
        "knowledge_set": "{\"study_cards\":{},\"lesson_history\":[]}",
        "updated_at": "2026-09-02T18:00:00Z"
    })
}

/// A stamp that is always in the future relative to any push a test can
/// make (`user_to_json` stamps with `Utc::now()`): the probe's `$gt` is a
/// lexicographic string comparison, so a bumped stamp must sort past
/// whatever the priming push wrote. Replaces far-future literals — a
/// hardcoded "tomorrow" is a date bomb that silently degrades the test
/// after it passes (the 2026-09-21 incident).
fn future_stamp() -> String {
    (chrono::Utc::now() + chrono::Duration::days(365)).to_rfc3339()
}

/// Minimal [`SavedWireRow`] for checkpoint-push tests: the bookkeeping
/// only reads `record_id` and the wire-bumped `updated_at`.
fn saved_wire_row(record_id: i64) -> SavedWireRow {
    SavedWireRow {
        record_id,
        wire: json!({ "updated_at": future_stamp() }),
    }
}

/// Syncs once so the meta records the current server fingerprint, then
/// returns the store for assertions.
async fn prime_synced_state(
    local: &SpyLocal,
    remote: &SpyRemote,
    meta: &MetaStore,
) -> origa::use_cases::SyncMeta {
    sync_merge(local, remote, meta).await.expect("priming sync");
    meta.load().await.expect("meta load")
}

// ═══════════════════════════════════════════════════════════════════════
// Steady state: the short-circuit
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn unchanged_state_skips_by_delta_probe_without_the_full_download() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    let primed = futures::executor::block_on(prime_synced_state(&local, &remote, &meta));
    assert!(!primed.dirty, "priming must clear the dirty flag");

    // Act: a second sync with no changes anywhere.
    let fetches_before = *remote.fetches.lock().unwrap();
    let pushes_before = remote.pushes.lock().unwrap().len();
    let saves_before = local.save_count();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("steady-state sync");

    // Assert: one delta probe, ZERO full fetches, zero writes anywhere.
    assert_eq!(*remote.probes.lock().unwrap(), 1, "exactly one probe");
    assert_eq!(
        *remote.fetches.lock().unwrap(),
        fetches_before,
        "the steady state must not download the row"
    );
    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        pushes_before,
        "steady state must not PATCH"
    );
    assert_eq!(
        local.save_count(),
        saves_before,
        "steady state must not write the local user"
    );
    assert!(remote.creates.lock().unwrap().is_empty());
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert_eq!(stored.probe_skips_since_full, 1, "the skip is counted");
}

// ═══════════════════════════════════════════════════════════════════════
// Full path scenarios
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn first_sync_is_full_and_records_server_fingerprint() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("first sync");

    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(!stored.dirty, "successful full sync clears dirty");
    assert!(stored.last_synced_fingerprint.is_some());

    // The recorded fingerprint must match what the server currently holds:
    // a follow-up sync skips via the probe without any full fetch.
    let fetches = *remote.fetches.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(
        *remote.fetches.lock().unwrap(),
        fetches,
        "second sync must only probe (empty answer), not fetch"
    );
    assert_eq!(*remote.probes.lock().unwrap(), 1);
}

#[test]
fn dirty_local_takes_full_path() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    // A user action dirties the meta.
    futures::executor::block_on(async {
        let mut m = meta.load().await.unwrap();
        m.mark_dirty();
        meta.store(&m).await.unwrap();
    });

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("full sync");

    assert!(
        !remote.pushes.lock().unwrap().is_empty(),
        "dirty local must be pushed"
    );
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(!stored.dirty);
}

#[test]
fn remote_change_takes_full_path() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    // Another device changes the remote row's content — a compliant writer
    // bumps the `updated_at` stamp with it (the wire bump in user_to_json).
    // The bumped stamp is generated a year out: it must sort past the
    // priming push's own stamp no matter when the test runs.
    {
        let mut rows = remote.rows.lock().unwrap();
        rows[0]["username"] = json!("changed-elsewhere");
        rows[0]["updated_at"] = json!(future_stamp());
    }

    let pushes_before = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("full sync");
    assert!(remote.pushes.lock().unwrap().len() > pushes_before);
}

#[test]
fn no_remote_row_creates_from_local_and_records_fingerprint() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("create sync");

    assert_eq!(remote.creates.lock().unwrap().len(), 1);
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(stored.last_synced_fingerprint.is_some());

    // Second sync: remote now matches the last sync → the probe answers
    // empty and no full fetch runs.
    let fetches = *remote.fetches.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(*remote.fetches.lock().unwrap(), fetches);
    assert_eq!(*remote.probes.lock().unwrap(), 1);
    assert_eq!(remote.pushes.lock().unwrap().len(), 0);
}

#[test]
fn no_users_anywhere_is_a_noop() {
    let local = SpyLocal::default();
    let remote = SpyRemote::new(vec![]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("noop sync");
    assert_eq!(*remote.fetches.lock().unwrap(), 1);
    assert!(remote.creates.lock().unwrap().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Correctness details from the ADR-045 threat model
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn recorded_fingerprint_matches_the_server_read_back() {
    // Echo fidelity (ADR-060): the post-push fingerprint is reconstructed
    // from the pushed payload (skeleton ∪ wire ∪ id) instead of a
    // re-fetch. A stamp-only change (false probe alarm) must run the full
    // check and SKIP on the fingerprint — proving the reconstruction
    // hashes exactly what the server returns.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("first sync");
    let pushes_after_first = remote.pushes.lock().unwrap().len();

    // Stamp-only bump: content identical, probe fires, fingerprint must
    // match the stored (echoed) row.
    let bumped_stamp = future_stamp();
    {
        let mut rows = remote.rows.lock().unwrap();
        rows[0]["updated_at"] = json!(bumped_stamp.clone());
    }

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        pushes_after_first,
        "a stamp-only change must skip on the matching reconstructed fingerprint"
    );

    // The skip branch heals the probe stamp from the fetched row.
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert_eq!(
        stored.last_seen_updated_at.as_deref(),
        Some(bumped_stamp.as_str())
    );
}

#[test]
fn server_side_normalization_settles_through_the_valve() {
    // The server "normalizes" pushes by adding a column the client never
    // sends. The post-push reconstruction misses it (the pushed payload is
    // the only source), so the valve's periodic full check detects the
    // divergence, merges, re-pushes — and the SKELETON from that fetched
    // row carries the extra column, settling the next full check into a
    // skip. One extra full cycle, once, instead of a permanent loop.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    remote.normalize_on_save = true;
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("first sync");
    assert_eq!(remote.pushes.lock().unwrap().len(), 1);

    // PROBE_SKIP_FULL_CHECK_INTERVAL probe-skips, then the valve fires.
    for _ in 0..PROBE_SKIP_FULL_CHECK_INTERVAL {
        futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("probe-skip sync");
    }
    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        1,
        "probe skips must not push"
    );

    // The valve full check: fingerprint diverges (server_generated_field
    // is in the server row but not in the reconstruction) → merge + push.
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("valve sync");
    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        2,
        "the valve full check re-pushes once against the normalized row"
    );

    // The re-push's skeleton came from the fetched (normalized) row, so
    // the valve settles: another interval of skips, then a full check
    // that now SKIPS.
    for _ in 0..PROBE_SKIP_FULL_CHECK_INTERVAL {
        futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("probe-skip sync");
    }
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("settled valve");
    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        2,
        "after the skeleton carries the normalized column, the full check skips"
    );
}

#[test]
fn concurrent_mutation_during_sync_window_keeps_dirty() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();

    // A card rating lands while the sync's push is in flight: the hook
    // fires from inside `save_with_record_id`, i.e. inside the window
    // between epoch capture and record_sync.
    let meta_for_hook = meta.clone();
    remote.on_push = Some(Box::new(move || {
        meta_for_hook.mark_dirty_direct();
    }));

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("full sync");

    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(
        stored.dirty,
        "a mutation inside the sync window must survive record_sync (CAS)"
    );
}

#[test]
fn failed_push_keeps_dirty_for_next_sync() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    remote.fail_pushes = true;
    let meta = MetaStore::default();

    let result = futures::executor::block_on(sync_merge(&local, &remote, &meta));
    assert!(result.is_err(), "push failure must surface");

    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(stored.dirty, "meta stays dirty until a push succeeds");
}

#[test]
fn duplicate_server_rows_resolve_to_smallest_id() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut older = fixture_row("a@example.com");
    older["id"] = json!(9);
    older["username"] = json!("dup-older");
    let mut newer = fixture_row("a@example.com");
    newer["id"] = json!(3);
    newer["username"] = json!("dup-newer");
    let remote = SpyRemote::new(vec![older, newer]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("sync");
    assert_eq!(*remote.pushes.lock().unwrap(), vec![3]);

    // Settle: after the min-id push the recorded fingerprint matches the
    // min-id row, so a second sync probes once and skips instead of
    // flapping between the duplicate rows.
    let fetches = *remote.fetches.lock().unwrap();
    let pushes = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(*remote.fetches.lock().unwrap(), fetches);
    assert_eq!(*remote.probes.lock().unwrap(), 1);
    assert_eq!(remote.pushes.lock().unwrap().len(), pushes);
}

#[test]
fn missing_local_record_takes_full_path_despite_clean_fingerprint() {
    // Regression for the skip-path guard: a clean meta plus a matching
    // fingerprint must NOT skip when the local record is gone — the full
    // path is what re-seeds the local store (ADR-045).
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    // The local store is wiped (device storage loss / corruption cleanup).
    let saves_before = local.save_count();
    *local.state.lock().unwrap() = None;

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("restore sync");

    assert_eq!(
        local.save_count(),
        saves_before + 1,
        "the restore path must write the local record"
    );
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(!stored.dirty, "restore settles the sync state");
}

#[test]
fn restore_from_remote_does_not_push_back() {
    // Fresh device: no local record. The seeded content comes from the
    // fetched row, so the restore must not PATCH megabytes of identical
    // data back (ADR-045).
    let local = SpyLocal::default();
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("restore sync");

    assert!(remote.pushes.lock().unwrap().is_empty(), "no push-back");
    assert!(remote.creates.lock().unwrap().is_empty());
    assert_eq!(local.save_count(), 1);

    // And it settles: the restore recorded the probe fields, so a second
    // sync is a single cheap probe — no fetch, no push.
    let fetches = *remote.fetches.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(*remote.fetches.lock().unwrap(), fetches);
    assert_eq!(*remote.probes.lock().unwrap(), 1);
    assert!(remote.pushes.lock().unwrap().is_empty());
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(
        stored.probe_target().is_some(),
        "restore must record the probe fields"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Delta probe (ADR-045 Future work)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn changed_remote_row_costs_one_probe_and_no_post_push_refetch() {
    // A compliant writer changed content AND bumped the stamp: the probe
    // fires and RETURNS the row — sync_merge uses it directly, merges,
    // pushes, and records the bookkeeping from the pushed payload. The
    // post-push fingerprint re-fetch is gone (ADR-060): the changed path
    // now costs strictly less than before the probe existed.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    {
        let mut rows = remote.rows.lock().unwrap();
        rows[0]["username"] = json!("changed-elsewhere");
        rows[0]["updated_at"] = json!(future_stamp());
    }

    let fetches_before = *remote.fetches.lock().unwrap();
    let pushes_before = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("full sync");

    assert_eq!(*remote.probes.lock().unwrap(), 1, "one probe");
    assert_eq!(
        *remote.fetches.lock().unwrap(),
        fetches_before,
        "the probe-returned row is the only download — no post-push re-fetch"
    );
    assert!(remote.pushes.lock().unwrap().len() > pushes_before);
}

#[test]
fn false_probe_alarm_self_heals_via_the_skip_branch() {
    // Stamp bumped, content identical (fingerprint still matches): the
    // probe fires, the skip branch runs, heals `last_seen_updated_at` from
    // the row in hand — the NEXT trigger is a single cheap probe again
    // instead of a permanent alarm loop.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    // Only the stamp moves; the fingerprint-relevant content does not.
    // The bumped stamp is generated a year out: it must sort past the
    // priming push's own stamp no matter when the test runs.
    let bumped_stamp = future_stamp();
    {
        let mut rows = remote.rows.lock().unwrap();
        rows[0]["updated_at"] = json!(bumped_stamp.clone());
    }

    let fetches_before = *remote.fetches.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("alarm sync");
    assert_eq!(*remote.probes.lock().unwrap(), 1, "the alarm fired");
    assert_eq!(
        *remote.fetches.lock().unwrap(),
        fetches_before,
        "skip branch: the probe already carried the row, no fetch needed"
    );
    let healed = futures::executor::block_on(meta.load()).expect("meta");
    assert_eq!(
        healed.last_seen_updated_at.as_deref(),
        Some(bumped_stamp.as_str()),
        "the stamp must be healed from the row"
    );

    // Next trigger: back to the cheap probe skip.
    let probes = *remote.probes.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("settled sync");
    assert_eq!(*remote.probes.lock().unwrap(), probes + 1);
    assert_eq!(
        *remote.fetches.lock().unwrap(),
        fetches_before,
        "no full fetch after healing"
    );
}

#[test]
fn probe_error_fails_open_to_the_full_fetch() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    *remote.probe_answer.lock().unwrap() = Some(Err(OrigaError::NetworkError {
        url: "https://app.example.net".to_string(),
        reason: "simulated probe transport failure".to_string(),
    }));

    let fetches_before = *remote.fetches.lock().unwrap();
    let pushes_before = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("fail-open sync");

    assert_eq!(*remote.probes.lock().unwrap(), 1);
    assert_eq!(
        *remote.fetches.lock().unwrap(),
        fetches_before + 1,
        "a failed probe must fall back to the full fetch"
    );
    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        pushes_before,
        "nothing changed → fingerprint skip, no push"
    );
}

#[test]
fn probe_valve_forces_a_full_check_after_the_interval() {
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    // Exhaust the valve: PROBE_SKIP_FULL_CHECK_INTERVAL cheap skips.
    for _ in 0..PROBE_SKIP_FULL_CHECK_INTERVAL {
        futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("cheap skip");
    }
    let fetches_before = *remote.fetches.lock().unwrap();
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert_eq!(
        stored.probe_skips_since_full, PROBE_SKIP_FULL_CHECK_INTERVAL,
        "the counter reaches the valve"
    );

    // The next trigger ignores the probe's empty answer and runs the full
    // check; recording the row resets the counter.
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("valve sync");
    assert!(
        *remote.fetches.lock().unwrap() > fetches_before,
        "the valve must force the full fetch path"
    );
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert_eq!(stored.probe_skips_since_full, 0, "the valve resets");
}

#[test]
fn a_newer_stale_duplicate_row_does_not_fire_the_probe() {
    // Duplicate rows are a documented production pathology (legacy
    // create-races). The probe targets the exact min-id row it synced
    // with; a stale duplicate carrying a NEWER stamp must not keep the
    // probe firing forever.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut canonical = fixture_row("a@example.com");
    canonical["id"] = json!(3);
    let mut stale_dup = fixture_row("a@example.com");
    stale_dup["id"] = json!(9);
    stale_dup["username"] = json!("dup-legacy");
    stale_dup["updated_at"] = json!(future_stamp()); // must sort past any push
    let remote = SpyRemote::new(vec![canonical, stale_dup]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));
    assert_eq!(*remote.pushes.lock().unwrap(), vec![3], "primed on min-id");

    let fetches_before = *remote.fetches.lock().unwrap();
    let pushes_before = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("steady sync");

    assert_eq!(*remote.probes.lock().unwrap(), 1);
    assert_eq!(
        *remote.fetches.lock().unwrap(),
        fetches_before,
        "the newer duplicate must not trigger any full fetch"
    );
    assert_eq!(remote.pushes.lock().unwrap().len(), pushes_before);
}

#[test]
fn dirty_meta_never_probes() {
    // A dirty local must take the full path directly — probing would be
    // wasted work, the outcome is a push regardless.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));
    meta.mark_dirty_direct();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("full sync");

    assert_eq!(
        *remote.probes.lock().unwrap(),
        0,
        "dirty state must skip the probe entirely"
    );
    assert!(!remote.pushes.lock().unwrap().is_empty());
}

#[test]
fn save_landing_during_the_probe_survives_and_pushes_immediately() {
    // The regression the write-back reload guards: a card rated while the
    // probe is in flight (`save()` → `mark_dirty`, no sync gate) must NOT
    // be clobbered by the skip bookkeeping. Writing the pre-await meta
    // snapshot back would clear the dirty flag while the rating sits in
    // the local store — the probe would then answer "unchanged" forever
    // and the push would never happen. Expected behaviour: the reloaded
    // dirty state routes the sync into the full path, which pushes the
    // rating right away.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    let meta_for_hook = meta.clone();
    remote.on_probe = Some(Box::new(move || {
        meta_for_hook.mark_dirty_direct();
    }));

    let pushes_before = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta))
        .expect("full path after the in-flight save");

    assert_eq!(*remote.probes.lock().unwrap(), 1, "the probe ran");
    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        pushes_before + 1,
        "the in-flight save must be pushed by THIS sync, not deferred"
    );
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(!stored.dirty, "the push settles the state");

    // And the next trigger is back to the cheap probe skip.
    remote.on_probe = None;
    let probes = *remote.probes.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("settled sync");
    assert_eq!(*remote.probes.lock().unwrap(), probes + 1);
    assert_eq!(remote.pushes.lock().unwrap().len(), pushes_before + 1);
}

// ═══════════════════════════════════════════════════════════════════════
// In-flight gate (concurrent merge dedup)
// ═══════════════════════════════════════════════════════════════════════

/// Wraps a remote spy so its first `find_current_raw` yields once before
/// answering: the first gated merge parks mid-pass, letting the
/// single-threaded executor poll the second caller, which then hits the
/// contended mutex and waits. Without this, the first merge would run to
/// completion inside its first poll and the waiting branch would go
/// untested.
struct YieldOnceRemote {
    inner: SpyRemote,
    yielded: Mutex<bool>,
}

impl YieldOnceRemote {
    fn new(inner: SpyRemote) -> Self {
        Self {
            inner,
            yielded: Mutex::new(false),
        }
    }
}

/// Yields to the executor exactly once (pending on the first poll, ready on
/// the second) so the first gated merge parks mid-pass deterministically.
/// The pending branch MUST schedule its own wake — a `Pending` without
/// `wake_by_ref` is never polled again, deadlocking `block_on` (the first
/// CI run hung three hours on exactly this).
fn yield_once() -> impl Future<Output = ()> {
    let mut yielded = false;
    poll_fn(move |cx| {
        if yielded {
            std::task::Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    })
}

impl RemoteUserSource for YieldOnceRemote {
    async fn find_current_raw(&self) -> Result<Option<RemoteRow>, OrigaError> {
        if !*self.yielded.lock().unwrap() {
            *self.yielded.lock().unwrap() = true;
            yield_once().await;
        }
        self.inner.find_current_raw().await
    }

    async fn save_with_record_id(
        &self,
        record_id: i64,
        user: &User,
    ) -> Result<SavedWireRow, OrigaError> {
        self.inner.save_with_record_id(record_id, user).await
    }

    async fn create(&self, user: &User) -> Result<SavedWireRow, OrigaError> {
        self.inner.create(user).await
    }

    async fn fetch_if_changed(
        &self,
        record_id: i64,
        seen_updated_at: &str,
    ) -> Result<Option<RemoteRow>, OrigaError> {
        self.inner
            .fetch_if_changed(record_id, seen_updated_at)
            .await
    }
}

#[test]
fn gated_concurrent_merges_collapse_into_a_single_push() {
    // Both callers observe a dirty local (a lesson just ended) and intend
    // the full path. The gate serializes them: the winner pushes, the
    // waiter re-runs afterwards, finds a clean meta with a matching
    // fingerprint, and collapses to the skip path — the duplicate
    // PATCH upload disappears.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(async {
        let mut m = meta.load().await.unwrap();
        m.mark_dirty();
        meta.store(&m).await.unwrap();
    });

    let pushes = Arc::clone(&remote.pushes);
    let yield_remote = YieldOnceRemote::new(remote);
    let gate: SyncGate = Arc::new(futures::lock::Mutex::new(()));

    let (first, second) = futures::executor::block_on(async {
        let a = sync_merge_gated(&local, &yield_remote, &meta, &gate);
        let b = sync_merge_gated(&local, &yield_remote, &meta, &gate);
        futures::join!(a, b)
    });
    first.expect("first merge");
    second.expect("second merge");

    assert_eq!(
        pushes.lock().unwrap().len(),
        1,
        "the gate must collapse concurrent dirty merges into a single push"
    );
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(!stored.dirty, "both passes settled the sync state");
}

#[test]
fn gated_merge_after_a_fresh_local_save_takes_the_full_path() {
    // The waiter must not blindly skip: a save that landed while it waited
    // (dirty flag) forces its own full pass through the same gate.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();
    futures::executor::block_on(prime_synced_state(&local, &remote, &meta));

    let gate: SyncGate = Arc::new(futures::lock::Mutex::new(()));
    futures::executor::block_on(sync_merge_gated(&local, &remote, &meta, &gate))
        .expect("first pass skips");

    // A user action (card rated) lands right after the first pass.
    meta.mark_dirty_direct();
    let pushes_before = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge_gated(&local, &remote, &meta, &gate))
        .expect("second pass");

    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        pushes_before + 1,
        "the dirty pass behind the gate must push, not skip"
    );
}

#[test]
fn gated_waiter_pushes_a_save_that_lands_while_it_waits() {
    // The genuinely concurrent shape: caller B is parked on the gate while
    // caller A's pass is in flight, and a user save (mark_dirty) lands
    // inside A's push window. A's record_sync CAS observes the newer epoch
    // and leaves the flag set, so B — after acquiring the gate — must take
    // its own full path instead of skipping the just-synced state.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    let meta = MetaStore::default();

    // Fires once — inside A's push window — simulating exactly one user
    // save landing while B is parked on the gate (B's own push must not
    // re-fire it, or the state could never settle).
    let fired = Arc::new(Mutex::new(false));
    let meta_for_hook = meta.clone();
    remote.on_push = Some(Box::new(move || {
        let mut fired = fired.lock().unwrap();
        if !*fired {
            *fired = true;
            meta_for_hook.mark_dirty_direct();
        }
    }));

    let pushes = Arc::clone(&remote.pushes);
    let yield_remote = YieldOnceRemote::new(remote);
    let gate: SyncGate = Arc::new(futures::lock::Mutex::new(()));

    let (first, second) = futures::executor::block_on(async {
        let a = sync_merge_gated(&local, &yield_remote, &meta, &gate);
        let b = sync_merge_gated(&local, &yield_remote, &meta, &gate);
        futures::join!(a, b)
    });
    first.expect("first merge");
    second.expect("second merge");

    assert_eq!(
        pushes.lock().unwrap().len(),
        2,
        "the waiter must push the save that landed while it was parked on the gate"
    );
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(!stored.dirty, "the waiter's push settles the state");
}

// ═══════════════════════════════════════════════════════════════════════
// save_sync core: the explicit checkpoint push under the single retry
// ═══════════════════════════════════════════════════════════════════════

/// The registration killer (Yandex OAuth, reported 2026-09-20): the login
/// bootstrap's remote push (`save_sync`) ran bare — no retry, unlike the
/// merge right before it — so ONE transient edge flap (a mangled
/// empty-body proxy answer surfacing as `API error: Failed to parse
/// response: … expected value at line 1 column 1`) failed the whole
/// registration, while the identical manual retry minutes later
/// succeeded. The checkpoint push must survive one such failure through
/// the shared sync retry.
#[test]
fn checkpoint_push_survives_one_transient_failure() {
    let local = SpyLocal::with_user(fixture_user("new@yandex.ru"));
    let meta = MetaStore::default();
    let user = fixture_user("new@yandex.ru");

    let attempts = Arc::new(Mutex::new(0usize));
    let attempts_for_assert = Arc::clone(&attempts);
    let fail_first = Arc::new(Mutex::new(true));
    let push = move || {
        let attempts = Arc::clone(&attempts);
        let fail_first = Arc::clone(&fail_first);
        let error = OrigaError::RepositoryError {
            reason: "API error: Failed to parse response: Repository error: expected value at line 1 column 1"
                .to_string(),
        };
        async move {
            *attempts.lock().unwrap() += 1;
            let mut fail = fail_first.lock().unwrap();
            if *fail {
                *fail = false;
                return Err(error);
            }
            Ok(saved_wire_row(7))
        }
    };

    let result =
        futures::executor::block_on(save_sync_core_with_delay(0, &local, &meta, &user, push));

    assert!(
        result.is_ok(),
        "one transient push failure must be retried away: {result:?}"
    );
    assert_eq!(
        *attempts_for_assert.lock().unwrap(),
        2,
        "exactly one retry after the first failure"
    );
    assert_eq!(local.save_count(), 1, "the local write stays authoritative");
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(
        !stored.dirty,
        "the checkpoint records its fingerprint inline from the pushed payload (ADR-060)"
    );
    assert!(stored.last_synced_fingerprint.is_some());
}

/// A dead session is not transient: the retry gate must refuse a second
/// roundtrip (re-login is the only path), and the local write stays.
#[test]
fn expired_session_push_is_never_retried() {
    let local = SpyLocal::with_user(fixture_user("new@yandex.ru"));
    let meta = MetaStore::default();
    let user = fixture_user("new@yandex.ru");

    let attempts = Arc::new(Mutex::new(0usize));
    let attempts_for_assert = Arc::clone(&attempts);
    let push = move || {
        let attempts = Arc::clone(&attempts);
        async move {
            *attempts.lock().unwrap() += 1;
            Err::<SavedWireRow, _>(OrigaError::SessionExpired)
        }
    };

    let result =
        futures::executor::block_on(save_sync_core_with_delay(0, &local, &meta, &user, push));

    assert!(result.is_err(), "an expired session must surface");
    assert_eq!(
        *attempts_for_assert.lock().unwrap(),
        1,
        "a second roundtrip cannot fix an expired session"
    );
    assert_eq!(local.save_count(), 1, "offline-first: the local write ran");
}

/// The local write comes first and gates the push: a broken local store
/// must not ship the user to the server at all (the local record is what
/// the next merge merges against).
#[test]
fn local_save_failure_never_reaches_the_push() {
    let local = SpyLocal::with_user(fixture_user("new@yandex.ru")).with_failing_saves();
    let meta = MetaStore::default();
    let user = fixture_user("new@yandex.ru");

    let attempts = Arc::new(Mutex::new(0usize));
    let attempts_for_assert = Arc::clone(&attempts);
    let push = move || {
        let attempts = Arc::clone(&attempts);
        async move {
            *attempts.lock().unwrap() += 1;
            Ok(saved_wire_row(7))
        }
    };

    let result =
        futures::executor::block_on(save_sync_core_with_delay(0, &local, &meta, &user, push));

    assert!(result.is_err(), "the local failure must surface");
    assert_eq!(
        *attempts_for_assert.lock().unwrap(),
        0,
        "a failed local write must not push anywhere"
    );
}

/// Healthy path: one push, no retry, and the bookkeeping settles inline —
/// the next merge collapses to the cheap delta probe (ADR-060).
#[test]
fn healthy_checkpoint_pushes_once_and_records_inline() {
    let local = SpyLocal::default();
    let meta = MetaStore::default();
    let user = fixture_user("new@yandex.ru");

    let attempts = Arc::new(Mutex::new(0usize));
    let attempts_for_assert = Arc::clone(&attempts);
    let push = move || {
        let attempts = Arc::clone(&attempts);
        async move {
            *attempts.lock().unwrap() += 1;
            Ok(saved_wire_row(42))
        }
    };

    let result =
        futures::executor::block_on(save_sync_core_with_delay(0, &local, &meta, &user, push));

    assert!(result.is_ok());
    assert_eq!(
        *attempts_for_assert.lock().unwrap(),
        1,
        "no retry on success"
    );
    assert_eq!(local.save_count(), 1);
    let stored = futures::executor::block_on(meta.load()).expect("meta");
    assert!(
        !stored.dirty,
        "the fingerprint is recorded from the pushed payload"
    );
    assert!(stored.last_synced_fingerprint.is_some());
    assert!(stored.last_synced_record_id.is_some());
}
