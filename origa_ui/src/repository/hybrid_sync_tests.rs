//! Native tests for the sync orchestration core (`sync_merge`, ADR-045).
//!
//! The spies mirror the contracts of the real repositories without any
//! JavaScript: `SpyLocal` implements `UserRepository`, `SpyRemote`
//! implements `RemoteUserSource` while simulating a server that stores the
//! pushed row (and may "normalize" it — e.g. add columns — to prove the
//! recorded fingerprint is server-authoritative).

use std::future::Future;
use std::sync::{Arc, Mutex};

use futures::future::poll_fn;
use origa::domain::{OrigaError, User};
use origa::traits::UserRepository;
use serde_json::{Value, json};
use ulid::Ulid;

use super::{SyncGate, save_sync_core_with_delay, sync_merge, sync_merge_gated};
use crate::repository::hybrid_repository::LocalUserPresence;
use crate::repository::sync_meta_store::{InMemorySyncMetaStore, SyncMetaStore};
use crate::repository::trailbase_repository::{
    RemoteRow, RemoteUserSource, remote_row_from_value, user_to_json,
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
            normalize_on_save: false,
            fail_pushes: false,
            on_push: None,
        }
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

    fn store_pushed_user(&self, record_id: i64, user: &User) -> Result<(), OrigaError> {
        let trailbase_id = TRAILBASE_ID
            .with(|id| id.borrow().clone())
            .unwrap_or_else(|| "00000000-0000-0000-0000-000000000001".to_string());
        let mut body = user_to_json(user, &trailbase_id)?;
        body["id"] = json!(record_id);
        if self.normalize_on_save {
            body["server_generated_field"] = json!("normalized");
        }

        let mut rows = self.rows.lock().unwrap();
        rows.retain(|row| row.get("id").and_then(Value::as_i64) != Some(record_id));
        rows.push(body);
        Ok(())
    }
}

impl RemoteUserSource for SpyRemote {
    async fn find_current_raw(&self) -> Result<Option<RemoteRow>, OrigaError> {
        self.row_for_fetch()
    }

    async fn save_with_record_id(&self, record_id: i64, user: &User) -> Result<(), OrigaError> {
        if self.fail_pushes {
            return Err(OrigaError::RepositoryError {
                reason: "simulated push failure".to_string(),
            });
        }
        self.store_pushed_user(record_id, user)?;
        self.pushes.lock().unwrap().push(record_id);
        if let Some(hook) = &self.on_push {
            hook();
        }
        Ok(())
    }

    async fn create(&self, user: &User) -> Result<i64, OrigaError> {
        let record_id = 42;
        self.store_pushed_user(record_id, user)?;
        self.creates.lock().unwrap().push(user.id());
        Ok(record_id)
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
fn unchanged_state_performs_no_writes_and_single_fetch() {
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

    // Assert: one raw fetch, zero local saves, zero pushes.
    assert_eq!(*remote.fetches.lock().unwrap(), fetches_before + 1);
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
    // a follow-up sync with no changes skips.
    let fetches = *remote.fetches.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(
        *remote.fetches.lock().unwrap(),
        fetches + 1,
        "second sync must only fetch (skip), not push"
    );
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

    // Another device changes the remote row's content.
    {
        let mut rows = remote.rows.lock().unwrap();
        rows[0]["username"] = json!("changed-elsewhere");
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

    // Second sync: remote now matches the last sync → skip.
    let fetches = *remote.fetches.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(*remote.fetches.lock().unwrap(), fetches + 1);
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
fn recorded_fingerprint_is_server_authoritative_not_request_body() {
    // The server "normalizes" pushes by adding a column the client never
    // sends. If the fingerprint were derived from the request body, the
    // next sync would take the full path forever; being derived from the
    // re-fetched server bytes, the state settles into skip.
    let local = SpyLocal::with_user(fixture_user("a@example.com"));
    let mut remote = SpyRemote::new(vec![fixture_row("a@example.com")]);
    remote.normalize_on_save = true;
    let meta = MetaStore::default();

    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("first sync");

    let pushes = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(
        remote.pushes.lock().unwrap().len(),
        pushes,
        "second sync must skip despite server-side normalization"
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
    // min-id row, so a second sync skips instead of flapping between the
    // duplicate rows.
    let fetches = *remote.fetches.lock().unwrap();
    let pushes = remote.pushes.lock().unwrap().len();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(*remote.fetches.lock().unwrap(), fetches + 1);
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

    // And it settles: a second sync skips.
    let fetches = *remote.fetches.lock().unwrap();
    futures::executor::block_on(sync_merge(&local, &remote, &meta)).expect("second sync");
    assert_eq!(*remote.fetches.lock().unwrap(), fetches + 1);
    assert!(remote.pushes.lock().unwrap().is_empty());
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

    async fn save_with_record_id(&self, record_id: i64, user: &User) -> Result<(), OrigaError> {
        self.inner.save_with_record_id(record_id, user).await
    }

    async fn create(&self, user: &User) -> Result<i64, OrigaError> {
        self.inner.create(user).await
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
            Ok(())
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
        stored.dirty,
        "the checkpoint records no fingerprint — the next merge takes the full path"
    );
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
            Err(OrigaError::SessionExpired)
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
            Ok(())
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

/// Healthy path: one push, no retry, checkpoint settled dirty (the
/// fingerprint is deliberately left to the next merge).
#[test]
fn healthy_checkpoint_pushes_once_and_stays_dirty() {
    let local = SpyLocal::default();
    let meta = MetaStore::default();
    let user = fixture_user("new@yandex.ru");

    let attempts = Arc::new(Mutex::new(0usize));
    let attempts_for_assert = Arc::clone(&attempts);
    let push = move || {
        let attempts = Arc::clone(&attempts);
        async move {
            *attempts.lock().unwrap() += 1;
            Ok(())
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
        stored.dirty,
        "the fingerprint recording belongs to the merge"
    );
}
