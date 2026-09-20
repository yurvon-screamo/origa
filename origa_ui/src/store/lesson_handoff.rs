//! One-shot handoff of the post-lesson user snapshot from the lesson
//! complete screen to the home page's init read.
//!
//! "Go home" navigates immediately while the multi-megabyte IndexedDB read
//! the home page needs for its stats is still ahead. The complete screen
//! starts that read in parallel and parks the result here; the home init
//! consumes it (bounded wait) so the stats paint without a second
//! structural-clone of the user record.
//!
//! Single-threaded WASM makes the thread_local safe; the routes involved
//! (`/lesson`, `/home`) are never mounted simultaneously, so a context is
//! not an option. Consumption is take-once: the first home mount consumes
//! and clears, a timeout clears too — no residue can paint stale stats on
//! a later mount. Auth transitions clear the handoff so a snapshot can
//! never leak across accounts.

use origa::domain::User;

/// Poll step for the awaiting consumer (see [`wait_for_handoff`]).
pub(crate) const POLL_STEP_MS: u64 = 50;
/// Bound on how long the home init waits for the lesson screen's read
/// before falling back to its own. Rare fallback path: a read slower than
/// this on a weak device degrades to the double read the handoff exists to
/// avoid — accepted, bounded.
pub(crate) const POLL_DEADLINE_MS: u64 = 2000;

thread_local! {
    static HANDOFF: std::cell::RefCell<Option<User>> = const { std::cell::RefCell::new(None) };
}

/// Parks the snapshot for the next home mount. Overwrites any unconsumed
/// previous snapshot (a newer lesson wins).
pub(crate) fn store_handoff(user: User) {
    HANDOFF.with(|cell| *cell.borrow_mut() = Some(user));
}

/// Consumes the snapshot if present (take-once semantics).
pub(crate) fn take_handoff() -> Option<User> {
    HANDOFF.with(|cell| cell.borrow_mut().take())
}

/// Drops an unconsumed snapshot. Called on auth transitions (logout, login,
/// account deletion) so a snapshot never crosses accounts, and by the
/// consumer on its timeout — a leftover here would otherwise paint stale
/// stats on a later, unrelated home mount.
pub(crate) fn clear_handoff() {
    HANDOFF.with(|cell| *cell.borrow_mut() = None);
}

pub(crate) fn has_handoff() -> bool {
    HANDOFF.with(|cell| cell.borrow().is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::NativeLanguage;

    fn fixture_user(email: &str) -> User {
        User::new(email.to_string(), NativeLanguage::Russian, None)
    }

    #[test]
    fn take_consumes_the_snapshot_once() {
        clear_handoff();
        store_handoff(fixture_user("a@example.com"));
        assert!(take_handoff().is_some(), "first take returns the snapshot");
        assert!(take_handoff().is_none(), "take-once: nothing remains");
    }

    #[test]
    fn a_newer_lesson_overwrites_the_unconsumed_snapshot() {
        clear_handoff();
        store_handoff(fixture_user("old@example.com"));
        store_handoff(fixture_user("new@example.com"));
        let taken = take_handoff().expect("snapshot");
        assert_eq!(taken.email(), "new@example.com");
        assert!(take_handoff().is_none());
    }

    #[test]
    fn clear_drops_an_unconsumed_snapshot() {
        clear_handoff();
        store_handoff(fixture_user("a@example.com"));
        clear_handoff();
        assert!(take_handoff().is_none(), "clear must leave no residue");
    }

    #[test]
    fn empty_handoff_reports_absent() {
        clear_handoff();
        assert!(!has_handoff());
        store_handoff(fixture_user("a@example.com"));
        assert!(has_handoff());
        clear_handoff();
    }
}
