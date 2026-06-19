//! IndexedDB orchestration for the nil-user re-key.
//!
//! Houses the single read-write transaction that atomically reseats a legacy
//! nil-keyed user row under the canonical id (or merges it into an existing
//! canonical row) and then deletes the stale nil row. Kept in its own module
//! so the pure decision functions (`reseat_user_id`,
//! `merge_legacy_progress_into_canonical`) in the parent module stay
//! independently unit-testable without dragging in idb wiring.

use idb::TransactionMode;
use origa::domain::{OrigaError, User};
use ulid::Ulid;

use super::legacy_migration_idb::{delete_row, read_row, row_exists, write_row};
use super::{merge_legacy_progress_into_canonical, migration_err, reseat_user_id};
use crate::repository::file_repository::{STORE_NAME, open_database};

/// Read-modify-write the nil-keyed row into the canonical id inside one
/// transaction. Returns 1 when a legacy row was migrated, 0 when there was
/// nothing to re-key. The transaction commits only after both the canonical
/// write and the nil delete succeed, so concurrent readers never observe both
/// keys at once.
pub(super) async fn rekey_nil_rows_in_single_transaction(
    correct_id: Ulid,
) -> Result<usize, OrigaError> {
    let db = open_database().await?;
    let transaction = db
        .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
        .map_err(|e| migration_err("Migration transaction failed", e))?;
    let store = transaction
        .object_store(STORE_NAME)
        .map_err(|e| migration_err("Migration store failed", e))?;

    let legacy = read_nil_row(&store).await?;
    let Some(legacy) = legacy else {
        return Ok(0);
    };

    apply_rekey_strategy(&store, &legacy, correct_id).await?;
    delete_row(&store, Ulid::nil()).await?;

    transaction
        .await
        .map_err(|e| migration_err("Migration commit failed", e))?;
    Ok(1)
}

/// Decide between reseat (no canonical row) and merge (canonical row exists)
/// and write the resulting row under the canonical id. The decision is kept
/// inside the open transaction so a canonical row that appears between the
/// existence check and the write is still observed atomically.
async fn apply_rekey_strategy(
    store: &idb::ObjectStore,
    legacy: &User,
    correct_id: Ulid,
) -> Result<(), OrigaError> {
    if !row_exists(store, correct_id).await? {
        // No canonical row yet: simply reseat the legacy row under the
        // canonical key, preserving all accumulated progress verbatim.
        let reseated = reseat_user_id(legacy, correct_id);
        write_row(store, &reseated, correct_id).await?;
        return Ok(());
    }

    // A canonical row already exists (e.g. the user logged in on another
    // device before upgrading this one). The legacy nil-keyed row still holds
    // progress accumulated on this device; merge it into the canonical row
    // instead of discarding it. Discarding was the previous behaviour and
    // silently destroyed local progress.
    let canonical = read_row(store, correct_id).await?;
    match canonical {
        None => {
            // row_exists said yes but the read returned nothing — a race or
            // corruption. Fall back to reseating the legacy row so progress is
            // never silently dropped.
            tracing::warn!(
                "Canonical row vanished between count and read for {}; reseating legacy",
                correct_id
            );
            let reseated = reseat_user_id(legacy, correct_id);
            write_row(store, &reseated, correct_id).await?;
        },
        Some(canonical) => {
            let merged = merge_legacy_progress_into_canonical(&canonical, legacy);
            tracing::info!(
                "Merged legacy nil-keyed progress into canonical row for {}",
                correct_id
            );
            write_row(store, &merged, correct_id).await?;
        },
    }
    Ok(())
}

/// Read the user held under the nil key, but only if its stored id is actually
/// nil — the key is just a string, so a non-nil id means the row is already
/// canonical and there is nothing to migrate.
async fn read_nil_row(store: &idb::ObjectStore) -> Result<Option<User>, OrigaError> {
    let Some(user) = read_row(store, Ulid::nil()).await? else {
        return Ok(None);
    };
    if user.id() == Ulid::nil() {
        Ok(Some(user))
    } else {
        tracing::debug!("Nil-key slot holds a non-nil user, nothing to migrate");
        Ok(None)
    }
}
