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
        let results = join_all(fetches).await;
        // One severity-routed report per batch instead of one event per
        // chunk: a dead CDN drops all ~198 corpus chunks in parallel.
        report_batch_outcomes(&results, chunks_to_load.len());
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

/// How one chunk's fetch+cache round ended. The split matters for the
/// batch report: a fetch failure follows the network invariant
/// (online = incident, offline = degradation), a payload that arrived but
/// failed to parse/cache is a content bug and reports as `error!`
/// regardless of connectivity — the same rule the JLPT loader pins.
#[derive(Debug)]
enum ChunkOutcome {
    Loaded,
    FetchFailed(String),
    ParseFailed(String),
}

/// One chunk's data fetch + cache (+ best-effort precompute). The
/// in-flight claim is always released, success or failure. Returns the
/// outcome; per-chunk failures are logged at debug level and aggregated
/// by the caller (`load_phrase_details_batch_via`) via
/// [`report_batch_outcomes`] — one dead CDN must not produce one event
/// per chunk (uwuwu#1454 review).
async fn fetch_and_cache_chunk<P: CdnProvider>(
    provider: &P,
    chunk_id: u32,
    hash: &str,
) -> ChunkOutcome {
    let path = format!("phrases/data/p{:04}.json?v={}", chunk_id, hash);
    let outcome = match provider.fetch_text(&path).await {
        Ok(json) => {
            if let Err(e) = cache_phrase_details(chunk_id, &json) {
                tracing::debug!("Failed to cache chunk {}: {e}", chunk_id);
                ChunkOutcome::ParseFailed(format!("chunk {chunk_id}: {e}"))
            } else {
                load_phrase_precompute_chunk_via(provider, chunk_id).await;
                ChunkOutcome::Loaded
            }
        },
        Err(e) => {
            // Per-chunk logging stays at debug level: a dead CDN drops the
            // whole corpus in parallel (all ~198 chunks of a batch would pass
            // this branch), which would flood Sentry with near-identical
            // events. The batch aggregates one severity-routed report below.
            tracing::debug!(chunk_id = chunk_id, error = ?e, "Failed to load chunk {}", chunk_id);
            ChunkOutcome::FetchFailed(format!("chunk {chunk_id}: {e:?}"))
        },
    };
    unmark_data_chunk_inflight(chunk_id);
    outcome
}

/// The batch-level report for [`fetch_and_cache_chunk`] outcomes. Fetch
/// failures get one severity-routed event (the network invariant); parse
/// failures get one `error!` event regardless of connectivity (content
/// bug). Both carry the failing chunk ids and first error texts — a bare
/// count would make a 404 on specific chunks indistinguishable from a
/// network refusal (uwuwu#1454 review round 2).
fn report_batch_outcomes(outcomes: &[ChunkOutcome], total: usize) {
    let fetch_failed: Vec<&String> = outcomes
        .iter()
        .filter_map(|o| match o {
            ChunkOutcome::FetchFailed(detail) => Some(detail),
            _ => None,
        })
        .collect();
    let parse_failed: Vec<&String> = outcomes
        .iter()
        .filter_map(|o| match o {
            ChunkOutcome::ParseFailed(detail) => Some(detail),
            _ => None,
        })
        .collect();
    if !fetch_failed.is_empty() {
        crate::tracing_console::report_network_failure(&format!(
            "Phrase data chunk fetches failed ({}/{}): {}",
            fetch_failed.len(),
            total,
            summarize_first(&fetch_failed)
        ));
    }
    if !parse_failed.is_empty() {
        tracing::error!(
            "Phrase data chunk payloads failed to parse/cache ({}/{}): {}",
            parse_failed.len(),
            total,
            summarize_first(&parse_failed)
        );
    }
}

/// First few failure texts for a batch report — the ids and the leading
/// cause without flooding the event payload.
fn summarize_first(details: &[&String]) -> String {
    let mut parts: Vec<String> = details.iter().take(3).map(|s| (*s).clone()).collect();
    if details.len() > parts.len() {
        parts.push(format!("… and {} more", details.len() - parts.len()));
    }
    parts.join("; ")
}

/// The batch-report contract pinned live (uwuwu#1454 review round 2):
/// fetch failures follow the network invariant, parse failures are
/// `error!` regardless of connectivity, a clean batch emits nothing.
#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use crate::core::log_capture;
    use crate::core::tauri_mock::OnlineOverride;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn fetch_failed(n: u32) -> Vec<ChunkOutcome> {
        (0..n)
            .map(|i| ChunkOutcome::FetchFailed(format!("chunk {}: net down", i)))
            .collect()
    }

    #[wasm_bindgen_test]
    fn fetch_failures_online_are_one_error_event_with_details() {
        let _online = OnlineOverride::install(true);
        log_capture::install();
        log_capture::take_events();
        let mut outcomes = fetch_failed(2);
        outcomes.push(ChunkOutcome::Loaded);
        report_batch_outcomes(&outcomes, 3);
        let events = log_capture::take_events();
        assert_eq!(events.len(), 1, "one aggregated event, got {events:?}");
        assert_eq!(events[0].0, tracing::Level::ERROR);
        let m = &events[0].1;
        assert!(m.contains("Phrase data chunk fetches failed (2/3)"), "{m}");
        assert!(m.contains("chunk 0"), "carries the failing chunk id: {m}");
        assert!(m.contains("net down"), "carries the error text: {m}");
    }

    #[wasm_bindgen_test]
    fn fetch_failures_offline_are_one_warn_event() {
        let _offline = OnlineOverride::install(false);
        log_capture::install();
        log_capture::take_events();
        report_batch_outcomes(&fetch_failed(1), 1);
        let events = log_capture::take_events();
        assert_eq!(events.len(), 1, "one aggregated event, got {events:?}");
        assert_eq!(events[0].0, tracing::Level::WARN);
        assert!(events[0].1.contains("(offline)"), "{}", events[0].1);
    }

    #[wasm_bindgen_test]
    fn parse_failures_are_error_regardless_of_connectivity() {
        // Online: a payload that arrived but does not parse is a content
        // bug — error even though the network is fine.
        let _online = OnlineOverride::install(true);
        log_capture::install();
        log_capture::take_events();
        report_batch_outcomes(
            &[
                ChunkOutcome::Loaded,
                ChunkOutcome::ParseFailed("chunk 4: bad json".into()),
            ],
            2,
        );
        let events = log_capture::take_events();
        assert_eq!(events.len(), 1, "got {events:?}");
        assert_eq!(events[0].0, tracing::Level::ERROR);
        assert!(
            events[0].1.contains("failed to parse/cache"),
            "{}",
            events[0].1
        );
        assert!(events[0].1.contains("chunk 4: bad json"), "{}", events[0].1);

        // Offline: still an error — connectivity never excuses a content bug.
        let _offline = OnlineOverride::install(false);
        log_capture::take_events();
        report_batch_outcomes(&[ChunkOutcome::ParseFailed("chunk 9: bad json".into())], 1);
        let events = log_capture::take_events();
        assert_eq!(events.len(), 1, "got {events:?}");
        assert_eq!(events[0].0, tracing::Level::ERROR);
    }

    #[wasm_bindgen_test]
    fn clean_batch_emits_no_events() {
        let _online = OnlineOverride::install(true);
        log_capture::install();
        log_capture::take_events();
        report_batch_outcomes(&[ChunkOutcome::Loaded, ChunkOutcome::Loaded], 2);
        assert!(
            log_capture::take_events().is_empty(),
            "a fully loaded batch must stay silent"
        );
    }
}
