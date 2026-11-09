//! Current-user resolution contract for `FileSystemUserRepository` (GH #634):
//! `get_current_user` must return the record under the canonical id derived
//! from the active session — never "the first record in the store". The wasm
//! suite shares one IndexedDB across all tests, so any "first record"
//! semantics silently leaks other tests' users into the read.

#![cfg(all(target_arch = "wasm32", test))]

use origa::domain::{DailyLoad, JlptProgress, KnowledgeSet, NativeLanguage, User};
use origa::traits::UserRepository;
use std::collections::HashSet;
use ulid::Ulid;
use wasm_bindgen_test::*;

use crate::test_support::{SessionGuard, install_session_for_user, test_user};

wasm_bindgen_test_configure!(run_in_browser);

/// A user with a fixed ULID far in the past: models a leftover record of a
/// previous test/run, whose key always sorts before freshly minted ids
/// (ULID keys are lexicographically ordered by their timestamp).
fn leftover_user_with_old_ulid() -> User {
    // Timestamp a thousand seconds past the epoch: safely above the nil
    // key (never touched by the legacy migration) and always below any
    // real `Ulid::new()`.
    let ancient_id = Ulid::from_parts(1_000_000, 42);
    User::from_row(
        ancient_id,
        "leftover@origa.local".to_string(),
        "leftover".to_string(),
        JlptProgress::new(),
        NativeLanguage::Russian,
        None,
        KnowledgeSet::new(),
        chrono::Utc::now(),
        HashSet::new(),
        DailyLoad::default(),
        0,
    )
}

/// The session user is returned even when an older-keyed leftover record
/// (other tests' residue) sorts first in the shared store. Pre-fix, the
/// "first record" read returned the leftover here.
#[wasm_bindgen_test]
async fn get_current_user_returns_session_user_despite_leftover_records() {
    let repo = crate::repository::file_repository::FileSystemUserRepository::new();

    // Arrange
    let leftover = leftover_user_with_old_ulid();
    let mine = test_user("current-user-session");
    repo.save(&leftover).await.expect("leftover seeded");
    repo.save(&mine).await.expect("session user seeded");
    let _session: SessionGuard = install_session_for_user(&mine);

    // Act
    let current = repo.get_current_user().await.expect("get_current_user");

    // Assert
    let current = current.expect("session user resolved");
    assert_eq!(
        current.id(),
        mine.id(),
        "get_current_user must resolve the session user, not the first store record"
    );

    // Record-level cleanup: the store is shared across the suite.
    repo.delete(leftover.id()).await.expect("leftover removed");
    repo.delete(mine.id()).await.expect("session user removed");
}

/// No session means no current user — even when the store holds records a
/// "first record" fallback would happily return. Pins the `Ok(None)` half of
/// the contract: the read must never degrade to the store's key order when
/// the session is absent (pre-fix, the leftover here WAS returned).
#[wasm_bindgen_test]
async fn get_current_user_without_session_returns_none() {
    let repo = crate::repository::file_repository::FileSystemUserRepository::new();

    // Arrange: records present, no session at all (defensive clear — the
    // suite shares one page and one session cache).
    let leftover = leftover_user_with_old_ulid();
    repo.save(&leftover).await.expect("leftover seeded");
    crate::repository::session::clear_session();

    // Act
    let current = repo.get_current_user().await.expect("get_current_user");

    // Assert
    assert!(
        current.is_none(),
        "without a session there is no current user — a store record must not be guessed"
    );

    // Record-level cleanup.
    repo.delete(leftover.id()).await.expect("leftover removed");
}

/// A session whose `trailbase_id` does not decode to a valid canonical id is
/// the "bug state, not an identity": the read resolves to `Ok(None)` (with a
/// warning) instead of falling back to any store record (pre-fix, the
/// leftover here WAS returned — the malformed session was ignored entirely).
#[wasm_bindgen_test]
async fn get_current_user_with_undecodable_session_id_returns_none() {
    use crate::repository::session::TrailBaseSession;

    let repo = crate::repository::file_repository::FileSystemUserRepository::new();

    // Arrange: a record a fallback would return + a session with a
    // garbage trailbase_id (undecodes to nil — no canonical identity).
    let leftover = leftover_user_with_old_ulid();
    repo.save(&leftover).await.expect("leftover seeded");
    let session = TrailBaseSession {
        auth_token: "test-token".to_string(),
        refresh_token: String::new(),
        email: "garbage@origa.local".to_string(),
        trailbase_id: "not-a-valid-id".to_string(),
        record_id: None,
        expires_at: u64::MAX,
    };
    crate::repository::session::set_session(&session).expect("garbage session set");
    let _session: SessionGuard = SessionGuard;

    // Act
    let current = repo.get_current_user().await.expect("get_current_user");

    // Assert
    assert!(
        current.is_none(),
        "an undecodable session id resolves to no current user, never a store record"
    );

    // Record-level cleanup.
    repo.delete(leftover.id()).await.expect("leftover removed");
}

/// A record that fails to decode under the canonical key is treated like
/// absence (`Ok(None)` + warning), never an error: the sync restore branch
/// relies on re-seeding the store instead of surfacing a repository error.
#[wasm_bindgen_test]
async fn get_current_user_treats_corrupted_record_as_absent() {
    use idb::TransactionMode;

    use crate::repository::file_repository::{STORE_NAME, open_database, user_key};

    let repo = crate::repository::file_repository::FileSystemUserRepository::new();

    // Arrange: session installed, but the canonical key holds garbage
    // instead of a decodable user record.
    let mine = test_user("corrupted-record");
    let _session: SessionGuard = install_session_for_user(&mine);
    {
        let db = open_database().await.expect("open database");
        let tx = db
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .expect("read-write transaction");
        let store = tx.object_store(STORE_NAME).expect("object store");
        store
            .put(
                &wasm_bindgen::JsValue::from_str("}{ not a user record"),
                Some(&wasm_bindgen::JsValue::from_str(&user_key(mine.id()))),
            )
            .expect("put request")
            .await
            .expect("corrupted record stored");
    }

    // Act
    let current = repo.get_current_user().await.expect("get_current_user");

    // Assert: corrupted record reads as "no current user", not Err.
    assert!(
        current.is_none(),
        "a corrupted canonical record is absence, not a repository error"
    );

    // Record-level cleanup: delete removes by key without decoding.
    repo.delete(mine.id())
        .await
        .expect("corrupted record removed");
}
