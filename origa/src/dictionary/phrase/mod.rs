//! Phrase dictionary: index storage (owned JSON-built form or the
//! zero-copy archived view of the CDN blob), initialization entry points
//! and the lookup API consumed by the rest of the crate.

mod detail;
mod index;

use std::collections::HashSet;
use std::sync::OnceLock;

use ulid::Ulid;

pub use detail::{
    PhraseDetail, cache_phrase_details, get_cached_phrase_detail, get_phrase_text,
    get_phrase_translation, is_chunk_loaded,
};
pub use index::{
    ArchivedPhraseIndexBlob, IndexEntry, PhraseIndex, PhraseIndexBlob, access_phrase_blob,
    build_phrase_index_from_json, serialize_phrase_index_blob_to_rkyv,
};

use crate::domain::OrigaError;

/// Loaded phrase index storage. The CDN rkyv blob installs a zero-copy
/// archived view; the JSON fallback builds the owned index.
enum PhraseStore {
    Owned(PhraseIndex),
    Archived(&'static ArchivedPhraseIndexBlob),
}

static PHRASE_INDEX: OnceLock<PhraseStore> = OnceLock::new();

pub fn init_phrase_index(json: &str) -> Result<(), OrigaError> {
    // Idempotent and race-tolerant: without this guard, a second caller
    // (e.g. another test module in the same binary) would receive a
    // spurious "already initialized" error even though the desired end
    // state is already reached.
    if is_phrases_loaded() {
        return Ok(());
    }

    let index = PhraseIndex::from_json(json)?;
    // Another thread may have populated the slot between the check above
    // and this `set`. The first valid index wins; the parsed result is
    // intentionally discarded in that case.
    let _ = PHRASE_INDEX.set(PhraseStore::Owned(index));
    Ok(())
}

/// Install the zero-copy archived view of a CDN blob payload.
pub fn install_archived_phrase_index(
    view: &'static ArchivedPhraseIndexBlob,
) -> Result<(), OrigaError> {
    PHRASE_INDEX
        .set(PhraseStore::Archived(view))
        .map_err(|_| OrigaError::PhraseParseError {
            reason: "phrase index already initialized".to_string(),
        })
}

pub fn is_phrases_loaded() -> bool {
    PHRASE_INDEX.get().is_some()
}

/// Number of indexed phrases — the loader's progress counter. O(1) on both
/// storage forms (no entry cloning just to count).
pub fn phrase_index_len() -> usize {
    match PHRASE_INDEX.get() {
        Some(PhraseStore::Owned(index)) => index.len(),
        Some(PhraseStore::Archived(view)) => view.len(),
        None => 0,
    }
}

pub fn get_phrases_by_token(token: &str) -> Vec<IndexEntry> {
    match PHRASE_INDEX.get() {
        Some(PhraseStore::Owned(index)) => index
            .get_phrases_by_token(token)
            .into_iter()
            .cloned()
            .collect(),
        Some(PhraseStore::Archived(view)) => view.get_phrases_by_token(token),
        None => Vec::new(),
    }
}

pub fn get_chunk_id(id: &Ulid) -> Option<u32> {
    match PHRASE_INDEX.get() {
        Some(PhraseStore::Owned(index)) => index.get_entry(id).map(|e| e.chunk_id()),
        Some(PhraseStore::Archived(view)) => view.get_entry(id).map(|e| e.chunk_id()),
        None => None,
    }
}

pub fn get_index_entry(id: &Ulid) -> Option<IndexEntry> {
    match PHRASE_INDEX.get() {
        Some(PhraseStore::Owned(index)) => index.get_entry(id).cloned(),
        Some(PhraseStore::Archived(view)) => view.get_entry(id),
        None => None,
    }
}

/// All indexed phrases, one owned clone per entry. The full index is
/// materialized up front (both storage forms) — callers iterating only a
/// prefix still pay the one-pass clone cost; current consumers (the
/// startup seeding pass, lesson builder fixtures) iterate everything.
pub fn iter_index_entries() -> Option<impl Iterator<Item = IndexEntry>> {
    PHRASE_INDEX.get().map(|store| match store {
        PhraseStore::Owned(index) => index
            .iter_entries()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter(),
        PhraseStore::Archived(view) => view.iter_entries().collect::<Vec<_>>().into_iter(),
    })
}

pub fn get_all_index_ids() -> HashSet<Ulid> {
    match PHRASE_INDEX.get() {
        Some(PhraseStore::Owned(index)) => index.all_ids().clone(),
        Some(PhraseStore::Archived(view)) => view.all_ids(),
        None => HashSet::new(),
    }
}

pub fn index_version() -> (u32, String) {
    match PHRASE_INDEX.get() {
        Some(PhraseStore::Owned(index)) => (index.version, index.hash.clone()),
        Some(PhraseStore::Archived(view)) => view.version(),
        None => (0, String::new()),
    }
}
