//! Vocabulary dictionary: loaded storage, initialization entry points and
//! the lookup API consumed by the rest of the crate.

mod blob;
mod chunks;
mod info;

use std::sync::OnceLock;

pub use blob::{
    ArchivedVocabularyBlob, VocabularyBlob, VocabularyDatabase, access_vocabulary_blob,
    serialize_vocabulary_blob_to_rkyv,
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

#[cfg(test)]
mod dispatch_tests {
    use super::*;

    fn chunk_data_with(chunk_01: &str) -> VocabularyChunkData {
        VocabularyChunkData {
            chunk_01: chunk_01.to_string(),
            chunk_02: "{}".to_string(),
            chunk_03: "{}".to_string(),
            chunk_04: "{}".to_string(),
            chunk_05: "{}".to_string(),
            chunk_06: "{}".to_string(),
            chunk_07: "{}".to_string(),
            chunk_08: "{}".to_string(),
            chunk_09: "{}".to_string(),
            chunk_10: "{}".to_string(),
            chunk_11: "{}".to_string(),
        }
    }

    /// Load through the shared test_support fixture: its call_once makes
    /// every caller converge on one install of the real chunks. Missing
    /// cdn/ data fails the fixture — callers bail out. Direct
    /// `init_vocabulary` calls here would race the fixture and poison its
    /// Once for every other test in the binary.
    fn ensure_vocabulary_loaded() {
        if !is_vocabulary_loaded() {
            crate::test_support::init_real_dictionaries();
        }
    }

    /// Parsing happens before the global slot write, so a broken chunk
    /// errors deterministically regardless of which test won the slot.
    #[test]
    fn init_rejects_broken_chunk_json_before_touching_the_slot() {
        // Act
        let Err(OrigaError::VocabularyParseError { reason }) =
            init_vocabulary(chunk_data_with("not json"))
        else {
            panic!("expected a parse error");
        };

        // Assert
        assert!(reason.contains("chunk_01"), "reason: {reason}");
    }

    #[test]
    fn init_from_rkyv_rejects_a_truncated_payload() {
        // Arrange — a fresh database serialized outside the global slot.
        let db = build_vocabulary_database_from_chunks(chunk_data_with(
            r#"{"猫": {"russian_translation": "кот", "english_translation": "cat"}}"#,
        ))
        .expect("valid chunks");
        let payload = serialize_vocabulary_blob_to_rkyv(&db).expect("serialize");

        // Act — losing the last byte pushes the rkyv root out of bounds.
        let Err(OrigaError::VocabularyParseError { reason }) =
            init_vocabulary_from_rkyv(&payload[..payload.len() - 1])
        else {
            panic!("expected an access error");
        };

        // Assert
        assert!(reason.contains("access"), "reason: {reason}");
    }

    /// Owned and archived dispatch must agree: serialize the installed
    /// store, access it zero-copy and compare the lookups word by word.
    /// The word key is a stable vocabulary entry, the translations are
    /// asserted only for cross-branch equality — no corpus content is
    /// hardcoded, so a dictionary refresh cannot break this unit.
    #[test]
    fn archived_view_matches_the_owned_store_for_lookups() {
        // Arrange
        ensure_vocabulary_loaded();
        if !is_vocabulary_loaded() {
            return; // cdn/ data absent — the real-dictionary fixture is unavailable
        }
        let word = "γ線";
        let language = &NativeLanguage::Russian;
        let owned_translations = get_translations(word, language);
        let Some(expected) = owned_translations.as_deref() else {
            return; // another test installed a dictionary without this word
        };
        assert!(!expected.is_empty(), "corpus entry carries no translations");

        // Act
        let bytes = serialize_vocabulary_to_rkyv().expect("serialize the owned store");
        let view = access_vocabulary_blob(&bytes).expect("access without install");

        // Assert — the archived branch answers exactly like the owned one.
        assert_eq!(view.translations(word, language), owned_translations);
        assert_eq!(
            view.translation(word, language),
            get_translation(word, language)
        );
        assert_eq!(
            view.description(word, language),
            get_description(word, language)
        );
    }

    /// Branch agreement on a synthetic database: the archived view answers
    /// exactly like the owned `VocabularyDatabase`, independent of the
    /// deployed corpus content.
    #[test]
    fn archived_view_matches_the_owned_database_for_lookups() {
        // Arrange
        let db = build_vocabulary_database_from_chunks(chunk_data_with(
            r#"{"猫": {"russian_translation": "кот", "english_translation": "cat"}}"#,
        ))
        .expect("valid chunks");
        let payload = serialize_vocabulary_blob_to_rkyv(&db).expect("serialize");
        let view = access_vocabulary_blob(&payload).expect("access without install");
        let language = &NativeLanguage::Russian;

        // Act + Assert — singular, plural and description agree on the hit;
        // a missing word is None on both branches.
        assert_eq!(
            view.translation("猫", language),
            db.get_translation("猫", language)
        );
        assert_eq!(
            view.translations("猫", language),
            db.get_translations("猫", language)
        );
        assert_eq!(
            view.description("猫", language),
            db.get_description("猫", language)
        );
        assert_eq!(view.translation("犬", language), None);
        assert_eq!(db.get_translation("犬", language), None);
    }

    #[test]
    fn lookups_before_initialization_yield_none() {
        // Best effort: only meaningful while the slot is still empty.
        if is_vocabulary_loaded() {
            return;
        }
        assert_eq!(get_translation("猫", &NativeLanguage::Russian), None);
        assert_eq!(get_translations("猫", &NativeLanguage::Russian), None);
        assert_eq!(get_description("猫", &NativeLanguage::Russian), None);
    }
}
