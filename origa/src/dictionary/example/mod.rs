//! Example dictionary: textbook example sentences for popular words
//! (issue #528). JSON storage mirrors `phrase`: an index (word -> sentence
//! refs) installed once, plus lazily loaded data chunks with sentence texts
//! and translations. rkyv twins are deferred until load budgets demand them.

mod detail;
mod index;

use std::sync::OnceLock;
use std::sync::RwLock;

pub use detail::{
    ExampleDetail, cache_example_details, get_cached_example_detail, is_example_chunk_loaded,
    reset_example_data_for_test,
};
pub use index::{ExampleIndex, ExampleRef};

use crate::domain::OrigaError;

static EXAMPLE_INDEX: OnceLock<RwLock<Option<ExampleIndex>>> = OnceLock::new();

fn store() -> &'static RwLock<Option<ExampleIndex>> {
    EXAMPLE_INDEX.get_or_init(|| RwLock::new(None))
}

/// Install the index from raw `cdn/examples/index.json` text.
/// Re-initialization is a no-op (idempotent, like the phrase index).
pub fn init_example_index(json: &str) -> Result<(), OrigaError> {
    let mut guard = store().write().map_err(|_| OrigaError::ExampleParseError {
        reason: "example index lock poisoned".to_string(),
    })?;
    if guard.is_some() {
        return Ok(());
    }
    let index = ExampleIndex::from_json(json)?;
    *guard = Some(index);
    Ok(())
}

pub fn is_examples_loaded() -> bool {
    store().read().map(|g| g.is_some()).unwrap_or(false)
}

/// Number of words with at least one example ref.
pub fn example_word_count() -> usize {
    let Ok(guard) = store().read() else {
        return 0;
    };
    match guard.as_ref() {
        Some(index) => index.word_count(),
        None => 0,
    }
}

/// Number of deduplicated sentences behind the index.
pub fn example_sentence_count() -> u32 {
    let Ok(guard) = store().read() else {
        return 0;
    };
    match guard.as_ref() {
        Some(index) => index.sentence_count(),
        None => 0,
    }
}

/// Test-only reset; keeps tests independent from global state (mirrors
/// `phrase::detail::reset_phrase_data_for_test`).
pub fn reset_example_index_for_test() {
    if let Ok(mut guard) = store().write() {
        *guard = None;
    }
}

/// Sentence refs for one word, empty when the word has no examples.
pub fn get_word_example_refs(word: &str) -> Vec<ExampleRef> {
    let Ok(guard) = store().read() else {
        return Vec::new();
    };
    match guard.as_ref() {
        Some(index) => index.get_refs(word),
        None => Vec::new(),
    }
}

/// Data chunk id holding a sentence: sentences are split into fixed-size
/// chunks by id (build script writes CHUNK_SIZE sentences per file).
pub fn chunk_id_for_sentence(sentence_id: u32) -> u32 {
    sentence_id / detail::CHUNK_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEX_JSON: &str = r#"{
        "v": 1,
        "h": "abc",
        "s": 3,
        "words": {
            "家族": {"refs": [[0, 0, 2], [2, 3, 5]]},
            "忙しい": {"refs": [[1, 3, 7]]}
        }
    }"#;

    #[test]
    fn init_is_idempotent() {
        reset_example_index_for_test();
        init_example_index(INDEX_JSON).expect("first install");
        init_example_index(INDEX_JSON).expect("second install is a no-op");
        assert!(is_examples_loaded());
        assert_eq!(example_word_count(), 2);
    }

    #[test]
    fn lookup_returns_refs_in_index_order() {
        reset_example_index_for_test();
        init_example_index(INDEX_JSON).expect("install");
        let refs = get_word_example_refs("家族");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].sentence_id(), 0);
        assert_eq!(refs[0].start(), 0);
        assert_eq!(refs[0].end(), 2);
        assert_eq!(chunk_id_for_sentence(2050), 2);
    }

    #[test]
    fn unknown_word_is_empty() {
        reset_example_index_for_test();
        init_example_index(INDEX_JSON).expect("install");
        assert!(get_word_example_refs("存在しない").is_empty());
    }

    #[test]
    fn corrupt_json_is_an_error() {
        reset_example_index_for_test();
        assert!(init_example_index("{not json").is_err());
    }

    /// Test-only reset; keeps tests independent from global state.
    pub fn reset_example_index_for_test() {
        if let Ok(mut guard) = store().write() {
            *guard = None;
        }
    }
}
