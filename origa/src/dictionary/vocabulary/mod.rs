//! Vocabulary dictionary: loaded storage, initialization entry points and
//! the lookup API consumed by the rest of the crate.

mod blob;
mod chunks;
mod info;

use std::sync::OnceLock;

pub use blob::{
    ArchivedVocabularyBlob, VocabularyBlob, VocabularyDatabase, access_vocabulary_blob,
    serialize_vocabulary_blob_to_rkyv, vocabulary_database_from_blob_rkyv,
};
pub use chunks::{VocabularyChunkData, build_vocabulary_database_from_chunks};
pub use info::VocabularyInfo;

use crate::domain::{NativeLanguage, OrigaError};

/// Loaded vocabulary storage. The CDN rkyv blob installs a zero-copy
/// archived view over the blob bytes (no per-entry allocations); the
/// JSON-chunk fallback builds the owned database.
enum VocabularyStore {
    Owned(VocabularyDatabase),
    Archived(&'static ArchivedVocabularyBlob),
}

static VOCABULARY_DICTIONARY: OnceLock<VocabularyStore> = OnceLock::new();

pub fn init_vocabulary(data: VocabularyChunkData) -> Result<(), OrigaError> {
    let db = VocabularyDatabase::from_chunks(data)?;
    set_vocabulary_database(db)
}

/// Install an already-built database into the global slot. Used by the CDN
/// blob builder (via `from_chunks`) and by loader wrappers after rkyv
/// deserialization. Fails if another database is already installed.
pub fn set_vocabulary_database(db: VocabularyDatabase) -> Result<(), OrigaError> {
    VOCABULARY_DICTIONARY
        .set(VocabularyStore::Owned(db))
        .map_err(|_| OrigaError::VocabularyParseError {
            reason: "Failed to set vocabulary dictionary".to_string(),
        })
}

/// Install the zero-copy archived view of a CDN blob payload.
pub fn install_archived_vocabulary(
    view: &'static ArchivedVocabularyBlob,
) -> Result<(), OrigaError> {
    VOCABULARY_DICTIONARY
        .set(VocabularyStore::Archived(view))
        .map_err(|_| OrigaError::VocabularyParseError {
            reason: "Failed to set vocabulary dictionary".to_string(),
        })
}

/// Initialize vocabulary from cached rkyv bytes (fast path). The local
/// cache holds the deterministic CDN-blob form, consumed via zero-copy
/// access. Skips JSON parsing of 11 chunks (~35 MB text).
pub fn init_vocabulary_from_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    let view = blob::access_vocabulary_blob(bytes)?;
    install_archived_vocabulary(view)
}

/// Serialize the loaded store to the deterministic CDN-blob form for the
/// local Cache API copy. Only the JSON-chunk fallback path (which builds
/// an owned database) ever writes this cache; the archived fast path skips
/// client re-serialization entirely.
pub fn serialize_vocabulary_to_rkyv() -> Result<Vec<u8>, OrigaError> {
    match VOCABULARY_DICTIONARY.get() {
        Some(VocabularyStore::Owned(db)) => blob::serialize_vocabulary_blob_to_rkyv(db),
        Some(VocabularyStore::Archived(_)) => Err(OrigaError::VocabularyParseError {
            reason: "archived vocabulary is already the CDN blob form".to_string(),
        }),
        None => Err(OrigaError::VocabularyParseError {
            reason: "Vocabulary dictionary not loaded".to_string(),
        }),
    }
}

pub fn is_vocabulary_loaded() -> bool {
    VOCABULARY_DICTIONARY.get().is_some()
}

pub fn get_translation(word: &str, native_language: &NativeLanguage) -> Option<String> {
    match VOCABULARY_DICTIONARY.get()? {
        VocabularyStore::Owned(db) => db.get_translation(word, native_language),
        VocabularyStore::Archived(view) => view.translation(word, native_language),
    }
}

pub fn get_translations(word: &str, native_language: &NativeLanguage) -> Option<Vec<String>> {
    match VOCABULARY_DICTIONARY.get()? {
        VocabularyStore::Owned(db) => db.get_translations(word, native_language),
        VocabularyStore::Archived(view) => view.translations(word, native_language),
    }
}

pub fn get_description(word: &str, native_language: &NativeLanguage) -> Option<String> {
    match VOCABULARY_DICTIONARY.get()? {
        VocabularyStore::Owned(db) => db.get_description(word, native_language),
        VocabularyStore::Archived(view) => view.description(word, native_language),
    }
}
