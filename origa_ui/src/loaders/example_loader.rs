//! Examples loader: installs the word→refs index at startup and fetches
//! sentence data chunks lazily (mirrors `phrase_loader` + `phrase_data_loader`,
//! JSON-only — the index is small enough not to need an rkyv twin yet).

use futures::future::join_all;
use origa::dictionary::example::{
    ExampleDetail, cache_example_details, chunk_id_for_sentence, get_cached_example_detail,
    get_word_example_refs, init_example_index, is_example_chunk_loaded, is_examples_loaded,
};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cdn_provider;

const EXAMPLES_INDEX_PATH: &str = "examples/index.json";

/// Data chunks with a fetch currently in flight: concurrent consumers
/// (word page + lesson views) must share one fetch per chunk.
static INFLIGHT_CHUNKS: std::sync::RwLock<Option<std::collections::HashSet<u32>>> =
    std::sync::RwLock::new(None);

fn try_mark_chunk_inflight(chunk_id: u32) -> bool {
    let mut guard = INFLIGHT_CHUNKS.write().unwrap_or_else(|e| e.into_inner());
    guard
        .get_or_insert_with(std::collections::HashSet::new)
        .insert(chunk_id)
}

fn unmark_chunk_inflight(chunk_id: u32) {
    if let Ok(mut guard) = INFLIGHT_CHUNKS.write() {
        if let Some(set) = guard.as_mut() {
            set.remove(&chunk_id);
        }
    }
}

/// Startup: install the examples index. Idempotent.
pub async fn load_examples() -> Result<(), OrigaError> {
    if is_examples_loaded() {
        tracing::debug!("Examples already loaded");
        return Ok(());
    }
    let provider = cdn_provider();
    load_examples_via(provider).await
}

/// Resolve one sentence: from cache, otherwise fetch+install its chunk.
pub async fn load_example_detail(sentence_id: u32) -> Result<ExampleDetail, OrigaError> {
    if let Some(detail) = get_cached_example_detail(sentence_id) {
        return Ok(detail);
    }
    load_example_detail_via(cdn_provider(), sentence_id).await
}

/// Provider-parameterized index load (tests drive this with a mock).
pub async fn load_examples_via<P: CdnProvider>(provider: &P) -> Result<(), OrigaError> {
    if is_examples_loaded() {
        return Ok(());
    }
    let json = provider.fetch_text(EXAMPLES_INDEX_PATH).await?;
    init_example_index(&json)?;
    tracing::info!(
        "Examples index loaded: {} words / {} sentences",
        origa::dictionary::example::example_word_count(),
        origa::dictionary::example::example_sentence_count()
    );
    Ok(())
}

pub async fn load_example_detail_via<P: CdnProvider>(
    provider: &P,
    sentence_id: u32,
) -> Result<ExampleDetail, OrigaError> {
    if let Some(detail) = get_cached_example_detail(sentence_id) {
        return Ok(detail);
    }
    let chunk_id = chunk_id_for_sentence(sentence_id);
    if !is_example_chunk_loaded(chunk_id) && try_mark_chunk_inflight(chunk_id) {
        let path = format!("examples/data/s{chunk_id:04}.json");
        match provider.fetch_text(&path).await {
            Ok(json) => {
                if let Err(e) = cache_example_details(&json) {
                    unmark_chunk_inflight(chunk_id);
                    return Err(e);
                }
            },
            Err(e) => {
                unmark_chunk_inflight(chunk_id);
                return Err(e);
            },
        }
        unmark_chunk_inflight(chunk_id);
    }
    // Another task's fetch may still be in flight; the cache may lag.
    get_cached_example_detail(sentence_id).ok_or(OrigaError::ExampleParseError {
        reason: format!("sentence {sentence_id} not present in chunk {chunk_id}"),
    })
}

/// A word's example: resolved sentence + this word's highlight offsets
/// (a shared sentence carries per-word offsets in the index).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WordExample {
    pub detail: ExampleDetail,
    pub start: i32,
    pub end: i32,
}

/// Resolve every ref of a word; failed chunks become per-ref errors.
pub async fn load_word_examples(word: &str) -> Vec<Result<WordExample, OrigaError>> {
    let provider = cdn_provider();
    load_word_examples_via(provider, word).await
}

pub async fn load_word_examples_via<P: CdnProvider>(
    provider: &P,
    word: &str,
) -> Vec<Result<WordExample, OrigaError>> {
    let refs = get_word_example_refs(word);
    // Parallel resolution (mirrors `phrase_data_loader::load_phrase_details_batch_via`):
    // refs of one word often live in different chunks.
    let jobs = refs.iter().map(|r| async move {
        load_example_detail_via(provider, r.sentence_id())
            .await
            .map(|detail| WordExample {
                detail,
                start: r.start(),
                end: r.end(),
            })
    });
    join_all(jobs).await
}

#[cfg(test)]
#[path = "example_loader_tests.rs"]
mod tests;
