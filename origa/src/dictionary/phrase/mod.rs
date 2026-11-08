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
    get_phrase_translation, is_chunk_loaded, reset_phrase_data_for_test,
};
pub use index::{
    ArchivedPhraseIndexBlob, ArchivedPhraseIndexBlobV3, IndexEntry, PhraseIndex, PhraseIndexBlob,
    PhraseIndexBlobV3, access_phrase_blob, access_phrase_blob_v3, build_phrase_index_from_json,
    serialize_phrase_index_blob_to_rkyv, serialize_phrase_index_v3_blob_to_rkyv,
};

use crate::domain::OrigaError;

/// Loaded phrase index storage. A CDN rkyv blob installs a zero-copy
/// archived view (v3 — deduplicated/interned — preferred, v2 legacy);
/// the JSON fallback builds the owned index.
enum PhraseStore {
    Owned(PhraseIndex),
    Archived(&'static ArchivedPhraseIndexBlob),
    ArchivedV3(&'static ArchivedPhraseIndexBlobV3),
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

/// Install the zero-copy archived view of a v3 CDN blob payload (#535).
pub fn install_archived_phrase_index_v3(
    view: &'static ArchivedPhraseIndexBlobV3,
) -> Result<(), OrigaError> {
    PHRASE_INDEX
        .set(PhraseStore::ArchivedV3(view))
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
        Some(PhraseStore::ArchivedV3(view)) => view.len(),
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
        Some(PhraseStore::ArchivedV3(view)) => view.get_phrases_by_token(token),
        None => Vec::new(),
    }
}

pub fn get_chunk_id(id: &Ulid) -> Option<u32> {
    match PHRASE_INDEX.get() {
        Some(PhraseStore::Owned(index)) => index.get_entry(id).map(|e| e.chunk_id()),
        Some(PhraseStore::Archived(view)) => view.get_entry(id).map(|e| e.chunk_id()),
        Some(PhraseStore::ArchivedV3(view)) => view.get_entry(id).map(|e| e.chunk_id()),
        None => None,
    }
}

pub fn get_index_entry(id: &Ulid) -> Option<IndexEntry> {
    match PHRASE_INDEX.get() {
        Some(PhraseStore::Owned(index)) => index.get_entry(id).cloned(),
        Some(PhraseStore::Archived(view)) => view.get_entry(id),
        Some(PhraseStore::ArchivedV3(view)) => view.get_entry(id),
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
        PhraseStore::ArchivedV3(view) => view.iter_entries().collect::<Vec<_>>().into_iter(),
    })
}

pub fn get_all_index_ids() -> HashSet<Ulid> {
    match PHRASE_INDEX.get() {
        Some(PhraseStore::Owned(index)) => index.all_ids().clone(),
        Some(PhraseStore::Archived(view)) => view.all_ids(),
        Some(PhraseStore::ArchivedV3(view)) => view.all_ids(),
        None => HashSet::new(),
    }
}

pub fn index_version() -> (u32, String) {
    match PHRASE_INDEX.get() {
        Some(PhraseStore::Owned(index)) => (index.version, index.hash.clone()),
        Some(PhraseStore::Archived(view)) => view.version(),
        Some(PhraseStore::ArchivedV3(view)) => view.version(),
        None => (0, String::new()),
    }
}

#[cfg(test)]
mod dispatch_tests {
    use super::*;

    fn synthetic_index_json() -> &'static str {
        r#"{"v":1,"h":"test","total":2,"phrases":[{"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["hello","world"],"c":0},{"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["goodbye","world"],"c":1}]}"#
    }

    /// Load the CDN corpus; when the data is absent, fall back to a synthetic
    /// index so the dispatch assertions have something real to answer with.
    fn ensure_index_loaded() -> bool {
        if !is_phrases_loaded() {
            crate::test_support::init_phrase_index_from_cdn();
        }
        if !is_phrases_loaded() {
            let _ = init_phrase_index(synthetic_index_json());
        }
        is_phrases_loaded()
    }

    /// The loaded-gate documents the race-tolerance contract: once an index
    /// is installed, init accepts any payload without parsing it.
    #[test]
    fn init_returns_ok_without_parsing_when_index_is_already_loaded() {
        // Arrange
        if !ensure_index_loaded() {
            return; // no corpus and no slot — nothing to assert
        }

        // Act + Assert — even garbage JSON is accepted while loaded.
        assert!(init_phrase_index("not json").is_ok());
    }

    #[test]
    fn dispatch_lookups_serve_the_installed_index() {
        // Arrange
        if !ensure_index_loaded() {
            return;
        }

        // Assert — every free function routes through the installed store.
        let len = phrase_index_len();
        assert!(len > 0, "installed index is empty");

        let ids = get_all_index_ids();
        assert_eq!(ids.len(), len, "all_ids and the index length diverge");

        let (version, hash) = index_version();
        assert!(version > 0, "installed index has no version");
        assert!(!hash.is_empty(), "installed index has no hash");

        let first = iter_index_entries()
            .expect("index is loaded")
            .next()
            .expect("index is non-empty");
        let id = *first.id();

        let entry = get_index_entry(&id).expect("entry by id");
        assert_eq!(entry.id(), &id);
        assert_eq!(get_chunk_id(&id), Some(entry.chunk_id()));

        let token = entry
            .tokens()
            .first()
            .expect("entry carries at least one token");
        let by_token = get_phrases_by_token(token);
        assert!(
            by_token.iter().any(|e| e.id() == &id),
            "token lookup misses the entry that carries the token"
        );
    }
}
