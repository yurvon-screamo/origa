//! One-shot migration of pre-fix user records keyed under the nil ULID.
//!
//! Before the `uuid_to_ulid` fix, every local save was written to IndexedDB
//! under `user:00000000000000000000000000` because the JWT `sub` was parsed as
//! hex and collapsed to `Ulid::nil()`. After the fix the canonical id is
//! derived from the session, so new saves land on `user:{correct_id}` — but the
//! old nil-keyed record still holds the user's accumulated progress and would
//! be returned nondeterministically by `get_current_user` (which takes the
//! first row in the store).
//!
//! This module re-keys such legacy rows to the canonical id derived from the
//! active session and deletes the stale nil row, preserving progress across
//! the upgrade. The migration runs at most once per browser: a LocalStorage
//! gate flag is set after the first attempt (success or no-op), and the whole
//! re-key happens inside a single read-write transaction so concurrent reads
//! never observe both keys at once.

use chrono::Utc;
use gloo_storage::{LocalStorage, Storage};
use origa::domain::{OrigaError, User};
use ulid::Ulid;

use super::session::get_session;
use super::uuid_to_ulid;

#[path = "legacy_migration_idb.rs"]
mod legacy_migration_idb;
#[path = "legacy_migration_rekey.rs"]
mod legacy_migration_rekey;
use legacy_migration_rekey::rekey_nil_rows_in_single_transaction;

const MIGRATION_GATE_KEY: &str = "origa_nil_migration_v1";

pub(super) fn migration_err<E: std::fmt::Debug>(prefix: &str, e: E) -> OrigaError {
    OrigaError::RepositoryError {
        reason: format!("{prefix}: {e:?}"),
    }
}

/// Re-keys any user rows stored under the nil ULID to the id encoded in the
/// active session. Returns the number of rows migrated. Runs at most once per
/// browser via a LocalStorage gate; subsequent calls are a cheap no-op.
///
/// The gate is only set once a session is available, so a user whose JWT has
/// expired on first post-upgrade launch does not permanently strand a nil-keyed
/// record: the next authenticated run retries.
pub(crate) async fn migrate_nil_users_to_session_id() -> Result<usize, OrigaError> {
    if migration_already_done() {
        return Ok(0);
    }

    let correct_id = match resolve_canonical_id() {
        Some(id) => id,
        // No session yet: defer both the migration and the gate so the next
        // authenticated launch can re-key the legacy record.
        None => return Ok(0),
    };

    let migrated = rekey_nil_rows_in_single_transaction(correct_id).await?;

    mark_migration_done();
    if migrated > 0 {
        tracing::info!(
            "Migrated {} legacy user record(s) from nil key to {}",
            migrated,
            correct_id
        );
    }
    Ok(migrated)
}

fn migration_already_done() -> bool {
    LocalStorage::get::<bool>(MIGRATION_GATE_KEY).unwrap_or(false)
}

fn mark_migration_done() {
    if let Err(e) = LocalStorage::set(MIGRATION_GATE_KEY, true) {
        tracing::warn!("Failed to persist nil-user migration gate: {}", e);
    }
}

fn resolve_canonical_id() -> Option<Ulid> {
    let session = get_session()?;
    canonical_id_from_trailbase_id(&session.trailbase_id)
}

/// Pure projection of a session's `trailbase_id` onto the canonical `Ulid`.
/// Extracted from `resolve_canonical_id` so the encoding/empty/nil rules are
/// unit-testable without a browser session: callers that already hold a
/// session string can reuse this directly.
fn canonical_id_from_trailbase_id(trailbase_id: &str) -> Option<Ulid> {
    if trailbase_id.is_empty() {
        return None;
    }
    let id = uuid_to_ulid(trailbase_id);
    (id != Ulid::nil()).then_some(id)
}

/// Rebuild a legacy user row under the canonical id. The domain `User` has no
/// id setter (identity is owned by the remote), so the row is rebuilt through
/// `from_row` with the canonical id and every other field copied verbatim.
pub(super) fn reseat_user_id(legacy: &User, new_id: Ulid) -> User {
    User::from_row(
        new_id,
        legacy.email().to_string(),
        legacy.username().to_string(),
        legacy.jlpt_progress().clone(),
        *legacy.native_language(),
        legacy.telegram_user_id().copied(),
        legacy.knowledge_set().clone(),
        *legacy.updated_at(),
        legacy.imported_sets().clone(),
        *legacy.daily_load(),
        legacy.known_vocab_hash(),
    )
}

/// Merge progress accumulated under the legacy nil key into the canonical row.
///
/// Identity and profile fields (id, email, username, native_language,
/// telegram_user_id, daily_load) stay canonical because the remote is the
/// source of truth — `User::merge` is intentionally avoided here because it
/// overwrites email/username, which would let a stale local copy clobber the
/// canonical profile. Only the progress collections the user accumulated
/// locally (knowledge_set, imported_sets) are unioned in so nothing is lost.
pub(super) fn merge_legacy_progress_into_canonical(canonical: &User, legacy: &User) -> User {
    let mut merged_knowledge = canonical.knowledge_set().clone();
    merged_knowledge.merge(legacy.knowledge_set());

    let mut merged_imported_sets = canonical.imported_sets().clone();
    for set_id in legacy.imported_sets() {
        merged_imported_sets.insert(set_id.clone());
    }

    User::from_row(
        canonical.id(),
        canonical.email().to_string(),
        canonical.username().to_string(),
        canonical.jlpt_progress().clone(),
        *canonical.native_language(),
        canonical.telegram_user_id().copied(),
        merged_knowledge,
        Utc::now(),
        merged_imported_sets,
        *canonical.daily_load(),
        canonical.known_vocab_hash(),
    )
}

#[cfg(test)]
#[path = "legacy_migration_tests.rs"]
mod tests;
