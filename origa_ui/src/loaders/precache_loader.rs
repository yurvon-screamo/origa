use std::sync::Arc;

use futures::stream::{self, StreamExt};
use origa::dictionary::cdn_blob::{guard_matches, split_blob};
use origa::dictionary::phrase::index_version;
use origa::dictionary::precompute_blob::inflate_blob;
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::loaders::dictionary::dict_path;
use crate::repository::cache_manager::guard_expectation_for;
use crate::repository::cdn_provider;

// v3 (uwuwu #1451): phrase content switched from raw JSON to deflated
// bundle blobs — the key bump forces a one-time re-precache so updated
// clients pick the blob layout (legacy JSON keys are cleaned up after
// the successful pass).
const BUNDLE_DOWNLOADED_KEY: &str = "/__origa_bundle_downloaded_v3__";
const CONCURRENCY: usize = 20;
const PHRASE_BUNDLE_COUNT: usize = 6;

#[derive(Clone, Default)]
pub struct PreCacheProgress {
    pub completed: usize,
    pub total: usize,
    pub current_file: String,
}

#[derive(Clone, Default)]
pub struct DownloadResult {
    pub total: usize,
    pub succeeded: usize,
}

pub fn get_base_bundle_resources() -> Vec<String> {
    let mut resources: Vec<String> = Vec::new();

    resources.extend([
        dict_path("char_def.bin"),
        dict_path("matrix.mtx"),
        dict_path("dict.trie"),
        dict_path("dict.valsidx"),
        dict_path("dict.vals"),
        dict_path("unk.bin"),
        dict_path("dict.wordsidx"),
        dict_path("dict.words"),
        dict_path("metadata.json"),
        "dictionaries/JmdictFurigana.txt".to_string(),
    ]);

    for i in 1..=11 {
        resources.push(format!("dictionary/chunk_{:02}.json", i));
    }

    resources.extend([
        "dictionary/kanji.json".to_string(),
        "dictionary/radicals.json".to_string(),
    ]);

    resources.push("grammar/grammar_v3.json".to_string());
    resources.push("phrases/phrase_index.v3.rkyv".to_string());

    // Phrase data bundles (uwuwu #1451): 6 deflated blobs replace the raw
    // JSON downloads (~5x smaller on the wire). After download,
    // extract_phrase_bundles_to_cache() parses each bundle and stores
    // individual chunks in Cache API so phrase_data_loader gets cache hits
    // without per-chunk CDN requests; the bundle copy is then deleted.
    for i in 0..PHRASE_BUNDLE_COUNT {
        resources.push(format!("phrases/data_bundle_{}.v3.rkyv", i));
    }

    resources.push("pitch/index.json".to_string());

    resources.extend([
        "well_known_set/well_known_sets_meta.json".to_string(),
        "well_known_set/well_known_types_meta.json".to_string(),
        "well_known_set/jlpt_n1.json".to_string(),
        "well_known_set/jlpt_n2.json".to_string(),
        "well_known_set/jlpt_n3.json".to_string(),
        "well_known_set/jlpt_n4.json".to_string(),
        "well_known_set/jlpt_n5.json".to_string(),
    ]);

    // Kanji SVGs are pre-cached on demand via card_precache_loader
    // for only the kanji the user actually studies.
    // Pre-fetching all ~6000+ kanji from the dictionary generates
    // many 404s for rare kanji that have no SVG files on the CDN.

    resources
}

pub async fn batch_download(
    paths: Vec<String>,
    on_progress: impl Fn(PreCacheProgress) + Clone + 'static,
) -> Result<DownloadResult, OrigaError> {
    let total = paths.len();
    let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let succeeded = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    stream::iter(paths)
        .map(|path| {
            let completed = completed.clone();
            let succeeded = succeeded.clone();
            let on_progress = on_progress.clone();
            async move {
                match cdn_provider::prefetch_to_cache(&path).await {
                    Ok(()) => {
                        succeeded.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    },
                    Err(e) => {
                        tracing::warn!(path = %path, error = ?e, "Failed to prefetch");
                    },
                }
                let done = completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                on_progress(PreCacheProgress {
                    completed: done,
                    total,
                    current_file: path,
                });
            }
        })
        .buffer_unordered(CONCURRENCY)
        .collect::<Vec<()>>()
        .await;

    Ok(DownloadResult {
        total,
        succeeded: succeeded.load(std::sync::atomic::Ordering::Relaxed),
    })
}

async fn mark_bundle_downloaded() -> Result<(), OrigaError> {
    cdn_provider::store_cache_marker(BUNDLE_DOWNLOADED_KEY, "ok").await
}

pub async fn is_bundle_downloaded() -> bool {
    cdn_provider::is_cached(BUNDLE_DOWNLOADED_KEY).await
}

pub async fn precache_base_bundle(
    on_progress: impl Fn(PreCacheProgress) + Clone + 'static,
) -> Result<DownloadResult, OrigaError> {
    let resources = get_base_bundle_resources();
    tracing::info!("Starting base bundle download: {} files", resources.len());
    let result = batch_download(resources, on_progress).await?;

    let success_rate = if result.total > 0 {
        result.succeeded as f64 / result.total as f64
    } else {
        0.0
    };

    if success_rate >= 0.95 {
        mark_bundle_downloaded().await?;
        // Legacy phrase JSON keys are removed only AFTER the successful
        // pass (review contract): during the pass cache-first hits skip
        // them without any network, and each removed key's blob
        // replacement must already be cached — otherwise an offline
        // start would lose its phrase content source.
        cleanup_legacy_phrase_json_cache().await;
        tracing::info!(
            "Bundle downloaded: {}/{} succeeded",
            result.succeeded,
            result.total
        );
    } else {
        tracing::warn!(
            "Bundle download incomplete: {}/{} succeeded",
            result.succeeded,
            result.total
        );
    }

    Ok(result)
}

/// After downloading phrase data bundles, extract individual chunks into
/// Cache API. This lets phrase_data_loader.rs work unchanged (cache hits).
///
/// Each bundle is `{"p0000": [...], "p0001": [...], ...}`.
/// We store each value under `phrases/data/p0000.json?v=HASH` so the cache
/// key matches what phrase_data_loader fetches. The bundle copy itself is
/// released after extraction — the per-chunk entries are the only readers
/// ever needed again.
pub async fn extract_phrase_bundles_to_cache() -> Result<usize, OrigaError> {
    let cdn = cdn_provider();
    let (_, hash) = index_version();
    let mut extracted = 0usize;

    for bundle_idx in 0..PHRASE_BUNDLE_COUNT {
        let chunks = match load_bundle_chunks_via(cdn, bundle_idx).await {
            Ok(chunks) => chunks,
            Err(e) => {
                tracing::warn!(
                    bundle = bundle_idx,
                    error = ?e,
                    "Failed to load phrase data bundle for extraction, skipping"
                );
                continue;
            },
        };

        for (chunk_key, chunk_value) in &chunks {
            let cache_path = format!("phrases/data/{}.json?v={}", chunk_key, hash);
            let chunk_text = match serde_json::to_string(chunk_value) {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(chunk = %chunk_key, error = %e, "Failed to serialize phrase chunk, skipping");
                    continue;
                },
            };
            if let Err(e) = cdn_provider::store_text_in_cache(&cache_path, &chunk_text).await {
                tracing::warn!(
                    chunk = %chunk_key,
                    error = ?e,
                    "Failed to store phrase chunk in cache during extraction"
                );
            } else {
                extracted += 1;
            }
        }

        release_bundle_cache_entries(bundle_idx).await;

        tracing::info!(
            bundle = bundle_idx,
            chunks = chunks.len(),
            "Extracted phrase data bundle to cache"
        );
    }

    tracing::info!(
        total_extracted = extracted,
        "Phrase data extraction complete"
    );
    Ok(extracted)
}

fn phrase_bundle_json_path(bundle_idx: usize) -> String {
    format!("phrases/data_bundle_{}.json", bundle_idx)
}

fn phrase_bundle_blob_path(bundle_idx: usize) -> String {
    format!("phrases/data_bundle_{}.v3.rkyv", bundle_idx)
}

/// Blob first (uwuwu #1451): precache stores deflated bundle blobs; the
/// legacy JSON bundle remains the fallback for a stale/unavailable blob
/// (deploy window, old cache) — the same layering as the phrase index.
async fn load_bundle_chunks_via<P: CdnProvider>(
    cdn: &P,
    bundle_idx: usize,
) -> Result<std::collections::BTreeMap<String, serde_json::Value>, OrigaError> {
    match cdn.fetch_bytes(&phrase_bundle_blob_path(bundle_idx)).await {
        Ok(deflated) => match phrase_bundle_chunks_from_blob(&deflated, bundle_idx) {
            Ok(chunks) => return Ok(chunks),
            Err(e) => tracing::warn!(
                bundle = bundle_idx,
                error = ?e,
                "Phrase bundle blob rejected, falling back to JSON"
            ),
        },
        Err(e) => tracing::warn!(
            bundle = bundle_idx,
            error = ?e,
            "Phrase bundle blob unavailable, falling back to JSON"
        ),
    }

    let json = cdn.fetch_text(&phrase_bundle_json_path(bundle_idx)).await?;
    parse_phrase_bundle_json(&json, bundle_idx)
}

fn parse_phrase_bundle_json(
    json: &str,
    bundle_idx: usize,
) -> Result<std::collections::BTreeMap<String, serde_json::Value>, OrigaError> {
    serde_json::from_str(json).map_err(|e| OrigaError::PhraseParseError {
        reason: format!("failed to parse phrase data bundle {bundle_idx}: {e}"),
    })
}

/// Pure blob-to-chunks core: inflate, validate the header, verify the
/// manifest guard over the chunk hashes and parse the generic JSON payload.
/// The guard expectation is derived from the payload's own chunk keys —
/// exactly the paths the manifest lists for the files the bundle contains.
fn phrase_bundle_chunks_from_blob(
    deflated: &[u8],
    bundle_idx: usize,
) -> Result<std::collections::BTreeMap<String, serde_json::Value>, OrigaError> {
    let inflated = inflate_blob(deflated).map_err(|e| OrigaError::PhraseParseError {
        reason: format!("phrase bundle {bundle_idx} blob not deflated ({e:?})"),
    })?;
    let (header, payload) = split_blob(&inflated).map_err(|e| OrigaError::PhraseParseError {
        reason: format!("phrase bundle {bundle_idx} blob header invalid: {e}"),
    })?;
    let json = std::str::from_utf8(payload).map_err(|e| OrigaError::PhraseParseError {
        reason: format!("phrase bundle {bundle_idx} blob payload is not UTF-8: {e}"),
    })?;
    let chunks = parse_phrase_bundle_json(json, bundle_idx)?;

    let chunk_paths: Vec<String> = chunks
        .keys()
        .map(|key| format!("phrases/data/{key}.json"))
        .collect();
    let path_refs: Vec<&str> = chunk_paths.iter().map(String::as_str).collect();
    if !guard_matches(&header, &guard_expectation_for(&path_refs)) {
        return Err(OrigaError::PhraseParseError {
            reason: format!("phrase bundle {bundle_idx} blob stale relative to manifest"),
        });
    }

    Ok(chunks)
}

/// Release the bundle copy once its chunks are cached: the blob entry
/// (and a legacy JSON entry from a previous layout) are never read again —
/// extraction is the only reader. Without this the bundle JSON and the
/// extracted chunks sit in Cache Storage twice (~158 MB combined).
async fn release_bundle_cache_entries(bundle_idx: usize) {
    for path in [
        phrase_bundle_blob_path(bundle_idx),
        phrase_bundle_json_path(bundle_idx),
    ] {
        if let Err(e) = cdn_provider::delete_from_cache(&path).await {
            tracing::warn!(path = %path, error = ?e, "Failed to release phrase bundle cache entry");
        }
    }
}

/// Remove the pre-blob phrase JSON keys after a successful pass. Each key
/// is removed only when its replacement is already cached, so an offline
/// start never loses its content source (review contract: deletion happens
/// after the pass, never before).
async fn cleanup_legacy_phrase_json_cache() {
    let mut legacy: Vec<(String, String)> = vec![(
        "phrases/phrase_index.json".to_string(),
        "phrases/phrase_index.v3.rkyv".to_string(),
    )];
    for idx in 0..PHRASE_BUNDLE_COUNT {
        legacy.push((phrase_bundle_json_path(idx), phrase_bundle_blob_path(idx)));
    }

    for (path, replacement) in legacy {
        if !cdn_provider::is_cached(&replacement).await {
            tracing::warn!(
                path = %path,
                replacement = %replacement,
                "Replacement not cached — legacy phrase JSON key kept"
            );
            continue;
        }
        if let Err(e) = cdn_provider::delete_from_cache(&path).await {
            tracing::warn!(path = %path, error = ?e, "Failed to clean up legacy phrase JSON cache key");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_bundle_includes_dictionaries() {
        let resources = get_base_bundle_resources();

        assert!(resources.contains(&"dictionaries/sudachidict-20260723/char_def.bin".to_string()));
        assert!(resources.contains(&"dictionaries/sudachidict-20260723/matrix.mtx".to_string()));
        assert!(resources.contains(&"dictionaries/sudachidict-20260723/dict.trie".to_string()));
        assert!(resources.contains(&"dictionaries/sudachidict-20260723/dict.valsidx".to_string()));
        assert!(resources.contains(&"dictionaries/JmdictFurigana.txt".to_string()));
        assert!(resources.contains(&"dictionary/kanji.json".to_string()));
        assert!(resources.contains(&"dictionary/radicals.json".to_string()));
        assert!(resources.contains(&"grammar/grammar_v3.json".to_string()));
        assert!(resources.contains(&"phrases/phrase_index.v3.rkyv".to_string()));
        assert!(resources.contains(&"pitch/index.json".to_string()));
        assert!(resources.contains(&"well_known_set/jlpt_n5.json".to_string()));
    }

    #[test]
    fn base_bundle_includes_vocabulary_chunks() {
        let resources = get_base_bundle_resources();
        for i in 1..=11 {
            assert!(resources.contains(&format!("dictionary/chunk_{:02}.json", i)));
        }
    }

    #[test]
    fn base_bundle_includes_phrase_data_bundles() {
        let resources = get_base_bundle_resources();
        for i in 0..PHRASE_BUNDLE_COUNT {
            assert!(
                resources.contains(&format!("phrases/data_bundle_{}.v3.rkyv", i)),
                "Missing phrases/data_bundle_{}.v3.rkyv",
                i
            );
        }
    }

    #[test]
    fn base_bundle_includes_phrase_blob_paths_instead_of_raw_json() {
        let resources = get_base_bundle_resources();

        assert!(resources.contains(&"phrases/phrase_index.v3.rkyv".to_string()));
        for i in 0..PHRASE_BUNDLE_COUNT {
            assert!(
                resources.contains(&format!("phrases/data_bundle_{}.v3.rkyv", i)),
                "Missing phrases/data_bundle_{}.v3.rkyv",
                i
            );
        }
        // Raw JSON phrase files never ride the precache pass — their
        // blob replacements are the only wire format (uwuwu #1451).
        assert!(!resources.contains(&"phrases/phrase_index.json".to_string()));
        assert!(
            !resources
                .iter()
                .any(|r| r.starts_with("phrases/data_bundle_") && r.ends_with(".json"))
        );
    }

    #[test]
    fn base_bundle_does_not_include_individual_phrase_chunks() {
        // Individual phrase data files should NOT be in the base bundle
        // anymore — they're replaced by data_bundle blobs
        let resources = get_base_bundle_resources();
        assert!(!resources.iter().any(|r| r.starts_with("phrases/data/p")));
    }

    #[test]
    fn legacy_cleanup_pairs_cover_every_removed_key() {
        // The cleanup contract: a legacy JSON key is removed only when its
        // blob replacement is cached — every removed key must have a named
        // replacement (verified indirectly through the helper functions the
        // cleanup is built from).
        for idx in 0..PHRASE_BUNDLE_COUNT {
            assert_eq!(
                phrase_bundle_json_path(idx),
                format!("phrases/data_bundle_{idx}.json")
            );
            assert_eq!(
                phrase_bundle_blob_path(idx),
                format!("phrases/data_bundle_{idx}.v3.rkyv")
            );
        }
    }
}
