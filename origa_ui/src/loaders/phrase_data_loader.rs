use std::collections::HashSet;

use futures::future::join_all;
use origa::dictionary::phrase::{
    PhraseDetail, cache_phrase_details, get_cached_phrase_detail, get_chunk_id, index_version,
    is_chunk_loaded,
};
use origa::dictionary::precompute_blob::{access_precompute_deflated, install_precompute_view};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;
use ulid::Ulid;

use crate::repository::cdn_provider;

#[cfg(test)]
#[path = "phrase_data_loader_tests.rs"]
mod tests;

/// Phrase chunks whose precompute blob was already fetched (or attempted)
/// this session — a loaded blob stays installed for the process lifetime.
/// `Option` inside keeps the initializer const (`HashSet::new` is not).
static PRECOMPUTE_LOADED_CHUNKS: std::sync::RwLock<Option<std::collections::HashSet<u32>>> =
    std::sync::RwLock::new(None);

/// Data chunks with a fetch currently in flight (#540-В1): the visible
/// slice loads and the search-driven full load run concurrently and must
/// share one fetch per chunk instead of duplicating it. Entries are
/// removed on completion, so a failed chunk stays retryable.
static INFLIGHT_DATA_CHUNKS: std::sync::RwLock<Option<std::collections::HashSet<u32>>> =
    std::sync::RwLock::new(None);

/// Drops the attempted-chunk set (logout): the installed views are cleared
/// by `reset_precomputed_store`, and the next user must be able to
/// re-install views for the chunks their phrase pages load.
pub fn reset_phrase_precompute_chunks() {
    *PRECOMPUTE_LOADED_CHUNKS
        .write()
        .unwrap_or_else(|e| e.into_inner()) = None;
}

fn mark_precompute_chunk_attempted(chunk_id: u32) -> bool {
    let mut guard = PRECOMPUTE_LOADED_CHUNKS
        .write()
        .unwrap_or_else(|e| e.into_inner());
    guard
        .get_or_insert_with(std::collections::HashSet::new)
        .insert(chunk_id)
}

/// Claims the chunk for this task. `false` = another fetch is already in
/// flight (the caller skips — the cache will appear when it lands).
fn try_mark_data_chunk_inflight(chunk_id: u32) -> bool {
    let mut guard = INFLIGHT_DATA_CHUNKS
        .write()
        .unwrap_or_else(|e| e.into_inner());
    guard
        .get_or_insert_with(std::collections::HashSet::new)
        .insert(chunk_id)
}

/// Releases the claim. Called on success AND failure — a failed chunk
/// must stay retryable by the next batch.
fn unmark_data_chunk_inflight(chunk_id: u32) {
    let mut guard = INFLIGHT_DATA_CHUNKS
        .write()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(set) = guard.as_mut() {
        set.remove(&chunk_id);
    }
}

/// Fetches the chunk's precompute blob and installs its zero-copy view
/// into the lookup store. Best-effort by design (#521): a missing or
/// invalid blob logs a warning and the render falls back to live
/// tokenization — the phrase data itself is never blocked by it.
///
/// Provider-parameterized like the data path so unit tests cover the
/// whole chunk load without the wasm-bound global CDN provider.
async fn load_phrase_precompute_chunk_via<P: CdnProvider>(provider: &P, chunk_id: u32) {
    if !mark_precompute_chunk_attempted(chunk_id) {
        return;
    }

    let (_, hash) = index_version();
    let path = format!("phrases/precomputed/p{chunk_id:04}.rkyv?v={hash}");
    let blob = match provider.fetch_bytes(&path).await {
        Ok(blob) => blob,
        Err(e) => {
            tracing::warn!("phrase precompute blob unavailable (chunk {chunk_id}): {e:?}");
            return;
        },
    };
    match access_precompute_deflated(&blob) {
        Ok((view, _header)) => install_precompute_view(view),
        Err(e) => tracing::warn!("phrase precompute blob rejected (chunk {chunk_id}): {e:?}"),
    }
}

#[expect(dead_code, reason = "lazy-load for future tasks")]
pub async fn load_phrase_detail(phrase_id: Ulid) -> Result<PhraseDetail, OrigaError> {
    if let Some(detail) = get_cached_phrase_detail(&phrase_id) {
        return Ok(detail);
    }

    let chunk_id = get_chunk_id(&phrase_id).ok_or(OrigaError::PhraseNotFound { phrase_id })?;

    if !is_chunk_loaded(chunk_id) {
        let (_, hash) = index_version();
        let path = format!("phrases/data/p{:04}.json?v={}", chunk_id, hash);
        let cdn = cdn_provider();
        let json = cdn.fetch_text(&path).await?;
        cache_phrase_details(chunk_id, &json)?;
        load_phrase_precompute_chunk_via(cdn, chunk_id).await;
    }

    get_cached_phrase_detail(&phrase_id).ok_or(OrigaError::PhraseNotFound { phrase_id })
}

pub async fn load_phrase_details_batch(ids: &[Ulid]) -> Vec<Result<PhraseDetail, OrigaError>> {
    load_phrase_details_batch_via(cdn_provider(), ids).await
}

/// Provider-parameterized core of [`load_phrase_details_batch`]: fetch
/// each still-missing data chunk once (in-flight dedup across concurrent
/// batches), cache it, then answer every id from the cache. Chunks load
/// in parallel — the visible slice typically spans dozens of them.
pub async fn load_phrase_details_batch_via<P: CdnProvider>(
    provider: &P,
    ids: &[Ulid],
) -> Vec<Result<PhraseDetail, OrigaError>> {
    let mut chunks_to_load: HashSet<u32> = HashSet::new();
    for id in ids {
        if get_cached_phrase_detail(id).is_none()
            && let Some(chunk_id) = get_chunk_id(id)
            && !is_chunk_loaded(chunk_id)
            && try_mark_data_chunk_inflight(chunk_id)
        {
            chunks_to_load.insert(chunk_id);
        }
    }

    if !chunks_to_load.is_empty() {
        let (_, hash) = index_version();
        let mut fetches = Vec::with_capacity(chunks_to_load.len());
        for chunk_id in &chunks_to_load {
            fetches.push(fetch_and_cache_chunk(provider, *chunk_id, &hash));
        }
        join_all(fetches).await;
    }

    // Chunks another concurrent batch is fetching are skipped, and their
    // ids resolve from the cache if that batch lands first — otherwise
    // they answer Err (PhraseNotFound). This is safe for every caller:
    // the orphan classifier treats a not-yet-loaded chunk as transient,
    // and the UI layers avoid racing the search backfill against the
    // visible-slice load. The next batch retry covers the gap.
    ids.iter()
        .map(|id| get_cached_phrase_detail(id).ok_or(OrigaError::PhraseNotFound { phrase_id: *id }))
        .collect()
}

/// One chunk's data fetch + cache (+ best-effort precompute). The
/// in-flight claim is always released, success or failure.
async fn fetch_and_cache_chunk<P: CdnProvider>(provider: &P, chunk_id: u32, hash: &str) {
    let path = format!("phrases/data/p{:04}.json?v={}", chunk_id, hash);
    match provider.fetch_text(&path).await {
        Ok(json) => {
            if let Err(e) = cache_phrase_details(chunk_id, &json) {
                tracing::warn!("Failed to cache chunk {}: {e}", chunk_id);
            } else {
                load_phrase_precompute_chunk_via(provider, chunk_id).await;
            }
        },
        Err(e) => tracing::warn!("Failed to load chunk {}: {e}", chunk_id),
    }
    unmark_data_chunk_inflight(chunk_id);
}
