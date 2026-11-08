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
