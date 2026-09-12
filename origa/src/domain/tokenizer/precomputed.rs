//! Precomputed tokenization derivatives: a serializable twin of
//! [`TokenInfo`] plus the per-text store that keeps render paths off the
//! tokenizer dictionary.
//!
//! Tokenization depends only on the text (never on the user), so its
//! derivatives — furigana spans for [`crate::domain::furiganize_segments`]
//! and tokens for [`crate::domain::lookup_tokens_translations`] — can be
//! computed once, offline, and consumed on every render. The store is keyed
//! by the exact strings components pass down (a full phrase text, each of
//! its [`crate::domain::split_japanese_sentences`] sentences, a card word,
//! a markdown text node), so a lookup never needs re-tokenization.
//!
//! Contract invariant: an entry with **empty `furigana_spans`** is a miss
//! for the furiganize path (`furiganize_segments` falls through to the
//! live paths) and only serves token translations. Card-word entries use
//! this to provide tokens without masking the furigana-dictionary lookup
//! for words that carry no reading.

use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

use super::{PartOfSpeech, TokenInfo};
use crate::domain::furigana_annotator::AnnotatedSpan;

/// Serializable twin of a tokenizer [`TokenInfo`]. The SudachiDict token
/// schema always produces `phonological_base_form ==
/// phonological_surface_form`, so a single `reading` field is lossless.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct PrecomputedToken {
    pub surface: String,
    pub base: String,
    pub reading: String,
    pub pos: PartOfSpeech,
}

impl PrecomputedToken {
    pub fn from_token_info(token: &TokenInfo) -> Self {
        Self {
            surface: token.orthographic_surface_form().to_string(),
            base: token.orthographic_base_form().to_string(),
            reading: token.phonological_surface_form().to_string(),
            pos: token.part_of_speech().clone(),
        }
    }

    /// Rebuild the internal [`TokenInfo`] so translation lookups run the
    /// same pipeline as live tokenization.
    pub fn to_token_info(&self) -> TokenInfo {
        TokenInfo {
            orthographic_base_form: self.base.clone(),
            phonological_base_form: self.reading.clone(),
            orthographic_surface_form: self.surface.clone(),
            phonological_surface_form: self.reading.clone(),
            part_of_speech: self.pos.clone(),
        }
    }
}

/// Precomputed derivation of one exact text: furigana spans (input to
/// [`crate::domain::furiganize_segments`]) and tokens (input to
/// [`crate::domain::lookup_tokens_translations`]). `furigana_spans` may be
/// empty — see the module contract invariant.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct PrecomputedEntry {
    pub furigana_spans: Vec<AnnotatedSpan>,
    pub tokens: Vec<PrecomputedToken>,
}

/// Precomputed entries installed from user cards (hundreds of small
/// strings). CDN blob views live in
/// [`crate::dictionary::precompute_blob`] instead.
static OWNED_ENTRIES: RwLock<Option<HashMap<String, PrecomputedEntry>>> = RwLock::new(None);

/// Installs one owned entry, overwriting any previous entry for the same
/// text (latest install wins — re-created cards refresh their cache).
pub fn install_precomputed_entry(text: &str, entry: PrecomputedEntry) {
    let mut guard = OWNED_ENTRIES.write().unwrap_or_else(|e| e.into_inner());
    guard
        .get_or_insert_with(HashMap::new)
        .insert(text.to_string(), entry);
}

/// Installs many owned entries in one lock acquisition.
pub fn install_precomputed_entries(entries: impl IntoIterator<Item = (String, PrecomputedEntry)>) {
    let mut guard = OWNED_ENTRIES.write().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    for (text, entry) in entries {
        map.insert(text, entry);
    }
}

/// Looks the text up across owned entries and every installed zero-copy
/// blob view. Returns a cloned entry — callers treat it as a value.
pub fn lookup_precomputed(text: &str) -> Option<PrecomputedEntry> {
    if let Some(map) = OWNED_ENTRIES
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        && let Some(entry) = map.get(text)
    {
        return Some(entry.clone());
    }
    crate::dictionary::precompute_blob::lookup_in_installed_views(text)
}

/// Drops every owned entry (card caches of the logged-out user) and every
/// installed blob view. Called on logout so a next user never observes the
/// previous user's card precompute.
pub fn reset_precomputed_store() {
    *OWNED_ENTRIES.write().unwrap_or_else(|e| e.into_inner()) = None;
    crate::dictionary::precompute_blob::clear_installed_views();
}

/// Number of owned entries — observability for tests and logs.
pub fn owned_precomputed_entries_len() -> usize {
    OWNED_ENTRIES
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map_or(0, HashMap::len)
}

/// Global count of `tokenize_text` invocations. Production observability
/// for the #521 invariant: renders of precomputed content must not grow
/// this counter.
static TOKENIZE_CALLS: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn record_tokenize_call() {
    TOKENIZE_CALLS.fetch_add(1, Ordering::Relaxed);
}

/// Total number of live tokenizations performed by this process so far.
pub fn tokenize_call_count() -> usize {
    TOKENIZE_CALLS.load(Ordering::Relaxed)
}

/// Serializes tests that mutate the global precomputed store — shared by
/// this module's tests and the `furigana` module tests that install/reset
/// entries too. Parallel install/reset of the same slot is a race.
#[cfg(test)]
pub(crate) static STORE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(reading: Option<&str>) -> PrecomputedEntry {
        PrecomputedEntry {
            furigana_spans: reading
                .map(|r| AnnotatedSpan {
                    text: "食べる".to_string(),
                    reading: Some(r.to_string()),
                    reading_spans: vec![],
                })
                .into_iter()
                .collect(),
            tokens: vec![PrecomputedToken {
                surface: "食べる".to_string(),
                base: "食べる".to_string(),
                reading: "タベル".to_string(),
                pos: PartOfSpeech::Verb,
            }],
        }
    }

    #[test]
    fn token_roundtrip_preserves_all_fields() {
        // Arrange
        let token = PrecomputedToken {
            surface: "食べます".to_string(),
            base: "食べる".to_string(),
            reading: "タベマス".to_string(),
            pos: PartOfSpeech::Verb,
        };

        // Act
        let rebuilt = token.to_token_info();

        // Assert
        assert_eq!(rebuilt.orthographic_surface_form(), "食べます");
        assert_eq!(rebuilt.orthographic_base_form(), "食べる");
        assert_eq!(rebuilt.phonological_surface_form(), "タベマス");
        assert_eq!(*rebuilt.part_of_speech(), PartOfSpeech::Verb);
    }

    #[test]
    fn entry_serde_roundtrip_preserves_content() {
        // Arrange
        let entry = sample_entry(Some("タベル"));

        // Act
        let json = serde_json::to_string(&entry).unwrap();
        let restored: PrecomputedEntry = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(restored, entry);
    }

    #[test]
    fn installed_entry_is_found_by_exact_text() {
        let _guard = STORE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Arrange
        reset_precomputed_store();
        install_precomputed_entry("食べる", sample_entry(Some("タベル")));

        // Act
        let found = lookup_precomputed("食べる");

        // Assert
        assert_eq!(found, Some(sample_entry(Some("タベル"))));
        assert!(lookup_precomputed("食べます").is_none());
        reset_precomputed_store();
    }

    #[test]
    fn reset_clears_owned_entries() {
        let _guard = STORE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Arrange
        reset_precomputed_store();
        install_precomputed_entry("食べる", sample_entry(Some("タベル")));

        // Act
        reset_precomputed_store();

        // Assert
        assert_eq!(owned_precomputed_entries_len(), 0);
        assert!(lookup_precomputed("食べる").is_none());
    }

    #[test]
    fn reinstall_overwrites_the_previous_entry() {
        let _guard = STORE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Arrange
        reset_precomputed_store();
        install_precomputed_entry("食べる", sample_entry(Some("タベル")));

        // Act
        install_precomputed_entry("食べる", sample_entry(None));

        // Assert
        let found = lookup_precomputed("食べる").expect("entry must exist");
        assert!(found.furigana_spans.is_empty(), "latest install wins");
        reset_precomputed_store();
    }
}
