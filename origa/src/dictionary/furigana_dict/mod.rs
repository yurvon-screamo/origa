//! Furigana dictionary: loaded storage (owned text-built form or the
//! zero-copy archived view of a CDN blob payload), initialization entry
//! points and the dispatching lookup API.

mod parse;

use std::sync::OnceLock;

use rkyv::util::AlignedVec;

use crate::domain::OrigaError;

pub use parse::ArchivedFuriganaDictionary;
pub use parse::FuriganaDictionary;

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ReadingSpan {
    pub start_index: usize,
    pub end_index: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct FuriganaEntry {
    pub text: String,
    pub reading: String,
    pub reading_spans: Vec<ReadingSpan>,
}

/// Loaded furigana storage. The CDN rkyv blob installs a zero-copy
/// archived view over the payload bytes; the JmdictFurigana text fallback
/// (and tests) build the owned dictionary.
pub enum FuriganaStore {
    Owned(FuriganaDictionary),
    Archived(&'static ArchivedFuriganaDictionary),
}

impl FuriganaStore {
    /// All entries for an exact word match, cloned into the owned form at
    /// the boundary (a handful of small entries per call).
    pub fn lookup_word(&self, word: &str) -> Vec<FuriganaEntry> {
        match self {
            Self::Owned(dict) => dict.lookup_word(word),
            Self::Archived(view) => view.lookup_word(word),
        }
    }

    /// All entries whose text starts with `prefix`.
    pub fn lookup_prefixed(&self, prefix: &str) -> Vec<FuriganaEntry> {
        match self {
            Self::Owned(dict) => dict.lookup_prefixed(prefix),
            Self::Archived(view) => view.lookup_prefixed(prefix),
        }
    }
}

static FURIGANA_DICT: OnceLock<FuriganaStore> = OnceLock::new();

pub fn is_furigana_dict_loaded() -> bool {
    FURIGANA_DICT.get().is_some()
}

/// Build the dictionary from JmdictFurigana text without installing it into
/// the global slot. Used by the CDN blob builder and by loader tests.
pub fn build_furigana_dict_from_text(content: &str) -> Result<FuriganaDictionary, OrigaError> {
    FuriganaDictionary::from_text(content)
}

/// Install an already-built dictionary (text fallback) into the global
/// slot. Fails if another dictionary is already installed.
pub fn set_furigana_dict(dict: FuriganaDictionary) -> Result<(), OrigaError> {
    FURIGANA_DICT
        .set(FuriganaStore::Owned(dict))
        .map_err(|_| OrigaError::FuriganaError {
            reason: "Furigana dictionary already loaded".to_string(),
        })
}

/// Install the zero-copy archived view of a CDN blob payload.
pub fn install_archived_furigana(
    view: &'static ArchivedFuriganaDictionary,
) -> Result<(), OrigaError> {
    FURIGANA_DICT
        .set(FuriganaStore::Archived(view))
        .map_err(|_| OrigaError::FuriganaError {
            reason: "Furigana dictionary already loaded".to_string(),
        })
}

pub fn init_furigana_dict(content: &str) -> Result<(), OrigaError> {
    let dict = FuriganaDictionary::from_text(content)?;
    set_furigana_dict(dict)
}

pub fn get_furigana_dict() -> Option<&'static FuriganaStore> {
    FURIGANA_DICT.get()
}

/// Serialize a built dictionary to rkyv bytes (payload of a CDN blob).
pub fn serialize_furigana_dict_to_rkyv(dict: &FuriganaDictionary) -> Result<Vec<u8>, OrigaError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(dict)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| OrigaError::FuriganaError {
            reason: format!("failed to serialize furigana dictionary: {e}"),
        })
}

/// Zero-copy archived view of a blob payload.
///
/// The payload is copied once into an immortal `AlignedVec` — rkyv's
/// checked `access` requires properly aligned bytes and a CDN blob header
/// offset leaves the payload at an arbitrary alignment. See
/// `vocabulary::blob::access_vocabulary_blob` for the full rationale.
pub fn access_furigana_payload(
    payload: &[u8],
) -> Result<&'static ArchivedFuriganaDictionary, OrigaError> {
    let aligned: &'static AlignedVec = Box::leak(Box::new({
        let mut buffer = AlignedVec::new();
        buffer.extend_from_slice(payload);
        buffer
    }));
    rkyv::access::<ArchivedFuriganaDictionary, rkyv::rancor::Error>(aligned.as_slice()).map_err(
        |e| OrigaError::FuriganaError {
            reason: format!("failed to access furigana blob: {e}"),
        },
    )
}
