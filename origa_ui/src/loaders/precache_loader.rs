use std::sync::Arc;

use futures::stream::{self, StreamExt};
use origa::dictionary::cdn_blob::{self, guard_matches, split_blob};
use origa::dictionary::phrase::index_version;
use origa::dictionary::precompute_blob::inflate_blob;
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::loaders::data_loader::VOCABULARY_CHUNK_PATHS;
use crate::loaders::dictionary::dict_path;
use crate::repository::cache_manager::guard_expectation_for;
use crate::repository::cdn_provider;

// v3 (uwuwu #1451): phrase content switched from raw JSON to deflated
// bundle blobs. v4 (uwuwu #1484): dictionary/grammar/kanji/radicals/pitch
// precache also switched to deflated blobs — the key bump forces a
// one-time re-precache so updated clients pick the blob layout (legacy
// JSON keys are cleaned up after the successful pass).
const BUNDLE_DOWNLOADED_KEY: &str = "/__origa_bundle_downloaded_v4__";
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

    // Deflated blobs replace the raw JSON downloads (uwuwu #1484): the
    // wire format is invisible to the Sentry "Large HTTP payload" detector
    // and several times smaller (NFT-06). The JSON sources stay on the CDN
    // as fallback + released-client source; their precache keys are removed
    // by the legacy cleanup after a successful pass.
    resources.push("dictionary/vocabulary.rkyv".to_string());
    resources.push("dictionary/kanji.rkyv".to_string());
    resources.push("dictionary/radicals.rkyv".to_string());
    resources.push("grammar/grammar_v3.rkyv".to_string());
    resources.push("phrases/phrase_index.v3.rkyv".to_string());

    // Phrase data bundles (uwuwu #1451): 6 deflated blobs replace the raw
    // JSON downloads (~5x smaller on the wire). After download,
    // extract_phrase_bundles_to_cache() parses each bundle and stores
    // individual chunks in Cache API so phrase_data_loader gets cache hits
    // without per-chunk CDN requests; the bundle copy is then deleted.
    for i in 0..PHRASE_BUNDLE_COUNT {
        resources.push(format!("phrases/data_bundle_{}.v3.rkyv", i));
    }

    resources.push("pitch/index.rkyv".to_string());

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
        // Legacy JSON keys are removed only AFTER the successful pass
        // (review contract): during the pass cache-first hits skip them
        // without any network, and each removed key's blob replacement
        // must already be cached — otherwise an offline start would lose
        // its content source.
        cleanup_legacy_json_cache().await;
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
/// manifest guard and parse the generic JSON payload. The guard binds the
/// blob to phrases/data_bundle_N.json — a VERSIONED_FILES path that always
/// ships in the client manifest (review round 2: chunk-file paths do NOT —
/// phrases/data is a SYNC_DIR, so a chunk-based guard is a guaranteed
/// Mismatch and a silent ~80 MB JSON re-download on every precache pass).
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
    let source_path = phrase_bundle_json_path(bundle_idx);
    let expectation = guard_expectation_for(&[&source_path]);
    phrase_bundle_chunks_with_expectation(&header, &expectation, payload, bundle_idx)
}

/// Testable core: guard first, parse after — the process manifest is a
/// OnceLock the unit tests cannot prime per-case (phrase_loader pattern).
fn phrase_bundle_chunks_with_expectation(
    header: &cdn_blob::BlobHeader,
    expectation: &cdn_blob::GuardExpectation,
    payload: &[u8],
    bundle_idx: usize,
) -> Result<std::collections::BTreeMap<String, serde_json::Value>, OrigaError> {
    if !guard_matches(header, expectation) {
        return Err(OrigaError::PhraseParseError {
            reason: "phrase data bundle blob stale relative to manifest".to_string(),
        });
    }
    let json = std::str::from_utf8(payload).map_err(|e| OrigaError::PhraseParseError {
        reason: format!("phrase bundle {bundle_idx} blob payload is not UTF-8: {e}"),
    })?;
    parse_phrase_bundle_json(json, bundle_idx)
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

/// (stale JSON key, blob replacement) pairs for every precache family that
/// moved to blobs. The cleanup contract: a key is removed only when its
/// replacement is already cached — the pair list is the single source of
/// both sides. Offline safety nets beyond the blob: the local vocabulary
/// rkyv cache (data_loader slow path serializes into it) and the runtime
/// JSON re-fetch (online).
fn legacy_cleanup_pairs() -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = vec![(
        "phrases/phrase_index.json".to_string(),
        "phrases/phrase_index.v3.rkyv".to_string(),
    )];
    for idx in 0..PHRASE_BUNDLE_COUNT {
        pairs.push((phrase_bundle_json_path(idx), phrase_bundle_blob_path(idx)));
    }
    // Deflated-blob families (uwuwu #1484): vocabulary consumes the
    // chunk_01..11 JSON (the blob builder concatenates them), the rest
    // consume their single JSON source.
    for chunk_path in VOCABULARY_CHUNK_PATHS {
        pairs.push((chunk_path.to_string(), "dictionary/vocabulary.rkyv".to_string()));
    }
    pairs.push(("dictionary/kanji.json".to_string(), "dictionary/kanji.rkyv".to_string()));
    pairs.push(("dictionary/radicals.json".to_string(), "dictionary/radicals.rkyv".to_string()));
    pairs.push(("grammar/grammar_v3.json".to_string(), "grammar/grammar_v3.rkyv".to_string()));
    pairs.push(("pitch/index.json".to_string(), "pitch/index.rkyv".to_string()));
    pairs
}

/// Remove the pre-blob JSON keys after a successful pass. Each key is
/// removed only when its replacement is already cached, so an offline
/// start never loses its content source (review contract: deletion happens
/// after the pass, never before).
async fn cleanup_legacy_json_cache() {
    for (path, replacement) in legacy_cleanup_pairs() {
        if !cdn_provider::is_cached(&replacement).await {
            tracing::warn!(
                path = %path,
                replacement = %replacement,
                "Replacement not cached — legacy JSON key kept"
            );
            continue;
        }
        if let Err(e) = cdn_provider::delete_from_cache(&path).await {
            tracing::warn!(path = %path, error = ?e, "Failed to clean up legacy JSON cache key");
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
        assert!(resources.contains(&"dictionary/vocabulary.rkyv".to_string()));
        assert!(resources.contains(&"dictionary/kanji.rkyv".to_string()));
        assert!(resources.contains(&"dictionary/radicals.rkyv".to_string()));
        assert!(resources.contains(&"grammar/grammar_v3.rkyv".to_string()));
        assert!(resources.contains(&"phrases/phrase_index.v3.rkyv".to_string()));
        assert!(resources.contains(&"pitch/index.rkyv".to_string()));
        assert!(resources.contains(&"well_known_set/jlpt_n5.json".to_string()));
    }

    #[test]
    fn base_bundle_rides_blobs_instead_of_raw_json_for_every_converted_family() {
        // uwuwu #1484: dictionary chunks, grammar, kanji, radicals and
        // pitch precache their deflated blobs — the raw JSON files never
        // ride the precache pass (detector-invisible wire format, the
        // JSON sources stay on CDN as fallback + released-client source).
        let resources = get_base_bundle_resources();

        assert!(resources.contains(&"dictionary/vocabulary.rkyv".to_string()));
        assert!(!resources.contains(&"dictionary/kanji.json".to_string()));
        assert!(!resources.contains(&"dictionary/radicals.json".to_string()));
        assert!(!resources.contains(&"grammar/grammar_v3.json".to_string()));
        assert!(!resources.contains(&"pitch/index.json".to_string()));
        for chunk in VOCABULARY_CHUNK_PATHS {
            assert!(
                !resources.contains(&chunk.to_string()),
                "chunk {chunk} must not ride the precache pass"
            );
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
        // blob replacement is cached — every removable key must have a named
        // replacement, and no pair may point at a non-legacy key.
        let pairs = legacy_cleanup_pairs();
        let mut expected: Vec<(String, String)> = vec![(
            "phrases/phrase_index.json".to_string(),
            "phrases/phrase_index.v3.rkyv".to_string(),
        )];
        for idx in 0..PHRASE_BUNDLE_COUNT {
            expected.push((
                phrase_bundle_json_path(idx),
                phrase_bundle_blob_path(idx),
            ));
        }
        for chunk in VOCABULARY_CHUNK_PATHS {
            expected.push((chunk.to_string(), "dictionary/vocabulary.rkyv".to_string()));
        }
        expected.push(("dictionary/kanji.json".to_string(), "dictionary/kanji.rkyv".to_string()));
        expected.push((
            "dictionary/radicals.json".to_string(),
            "dictionary/radicals.rkyv".to_string(),
        ));
        expected.push((
            "grammar/grammar_v3.json".to_string(),
            "grammar/grammar_v3.rkyv".to_string(),
        ));
        expected.push(("pitch/index.json".to_string(), "pitch/index.rkyv".to_string()));

        assert_eq!(pairs, expected);
        // Every stale key is a JSON path — blobs are never cleanup targets.
        for (path, _) in &pairs {
            assert!(path.ends_with(".json"), "cleanup target must be JSON: {path}");
        }
    }

    /// Recording mock: serves canned bytes/text per path and tracks which
    /// paths were requested (external CDN boundary — a mock is appropriate;
    /// the same pattern as the phrase_loader tests next door).
    struct MockCdn {
        responses: std::cell::RefCell<Vec<(String, MockResponse)>>,
        requested: std::cell::RefCell<Vec<String>>,
    }

    enum MockResponse {
        Bytes(Vec<u8>),
        Text(String),
    }

    impl MockCdn {
        fn with_blob_and_json(blob: Option<Vec<u8>>, json: &str) -> Self {
            let mut responses = Vec::new();
            if let Some(bytes) = blob {
                responses.push((phrase_bundle_blob_path(0), MockResponse::Bytes(bytes)));
            }
            responses.push((
                phrase_bundle_json_path(0),
                MockResponse::Text(json.to_string()),
            ));
            Self {
                responses: std::cell::RefCell::new(responses),
                requested: std::cell::RefCell::new(Vec::new()),
            }
        }

        fn requested_paths(&self) -> Vec<String> {
            self.requested.borrow().clone()
        }

        fn find(&self, path: &str) -> Option<MockResponse> {
            self.responses
                .borrow()
                .iter()
                .find(|(candidate, _)| candidate == path)
                .map(|(_, response)| match response {
                    MockResponse::Bytes(bytes) => MockResponse::Bytes(bytes.clone()),
                    MockResponse::Text(text) => MockResponse::Text(text.clone()),
                })
        }
    }

    impl CdnProvider for MockCdn {
        fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
            self.requested.borrow_mut().push(path.to_string());
            let result = match self.find(path) {
                Some(MockResponse::Text(text)) => Ok(text),
                _ => Err(OrigaError::NetworkError {
                    url: path.to_string(),
                    reason: "not stubbed".to_string(),
                }),
            };
            std::future::ready(result)
        }

        fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
            self.requested.borrow_mut().push(path.to_string());
            let result = match self.find(path) {
                Some(MockResponse::Bytes(bytes)) => Ok(bytes),
                _ => Err(OrigaError::NetworkError {
                    url: path.to_string(),
                    reason: "not stubbed".to_string(),
                }),
            };
            std::future::ready(result)
        }
    }

    fn sample_chunks() -> std::collections::BTreeMap<String, serde_json::Value> {
        let mut chunks = std::collections::BTreeMap::new();
        chunks.insert(
            "p0000".to_string(),
            serde_json::json!({"i": "01A", "t": ["hello"], "c": 0}),
        );
        chunks.insert(
            "p0001".to_string(),
            serde_json::json!({"i": "01B", "t": ["world"], "c": 0}),
        );
        chunks
    }

    fn bundle_json(chunks: &std::collections::BTreeMap<String, serde_json::Value>) -> String {
        serde_json::to_string(chunks).unwrap()
    }

    fn hex_digest(data: &[u8]) -> String {
        origa::dictionary::cdn_blob::sha256_bytes(data)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    }

    /// Build a deflated bundle blob with the production guard wiring
    /// (manifest_guard = sha256 over the hex hash of the source JSON);
    /// `guard_hashes` overrides it to simulate a stale (deploy-window)
    /// blob bound to a foreign generation.
    fn build_bundle_blob(
        chunks: &std::collections::BTreeMap<String, serde_json::Value>,
        guard_hashes: Option<Vec<String>>,
    ) -> Vec<u8> {
        use origa::dictionary::cdn_blob::{
            BlobHeader, SCHEMA_VERSION, manifest_guard_from_hex_hashes, sha256_bytes,
        };
        use origa::dictionary::precompute_blob::deflate_blob;

        let json = bundle_json(chunks);
        let hexes: Vec<String> = guard_hashes.unwrap_or_else(|| vec![hex_digest(json.as_bytes())]);
        let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
        let header = BlobHeader {
            schema_version: SCHEMA_VERSION,
            source_sha256: sha256_bytes(json.as_bytes()),
            manifest_guard: manifest_guard_from_hex_hashes(&refs),
        };
        deflate_blob(&cdn_blob::build_blob(&header, json.as_bytes()))
    }

    /// The expectation the client derives in production from the fetched
    /// manifest for the bundle's source path (injectable form of
    /// `guard_expectation_for` — the OnceLock cannot be primed per-case).
    fn expectation_from_source_json(
        chunks: &std::collections::BTreeMap<String, serde_json::Value>,
    ) -> cdn_blob::GuardExpectation {
        let hex = hex_digest(bundle_json(chunks).as_bytes());
        cdn_blob::guard_expectation(Some(&[Some(hex)]))
    }

    #[tokio::test]
    async fn valid_bundle_blob_serves_extraction_without_touching_json() {
        // Arrange
        let chunks = sample_chunks();
        let provider = MockCdn::with_blob_and_json(
            Some(build_bundle_blob(&chunks, None)),
            &bundle_json(&chunks),
        );

        // Act
        let loaded = load_bundle_chunks_via(&provider, 0).await.unwrap();

        // Assert: the blob answers; the JSON fallback is never requested.
        assert_eq!(loaded, chunks);
        assert!(
            !provider
                .requested_paths()
                .contains(&phrase_bundle_json_path(0)),
            "blob path must serve extraction without the JSON fallback"
        );
    }

    #[tokio::test]
    async fn corrupted_bundle_blob_falls_back_to_json() {
        // Arrange: bytes that do not inflate — the shape of truncated/garbage.
        let chunks = sample_chunks();
        let provider = MockCdn::with_blob_and_json(Some(vec![0xff_u8; 64]), &bundle_json(&chunks));

        // Act
        let loaded = load_bundle_chunks_via(&provider, 0).await.unwrap();

        // Assert: JSON fallback serves the same chunks.
        assert_eq!(loaded, chunks);
        assert!(
            provider
                .requested_paths()
                .contains(&phrase_bundle_json_path(0))
        );
    }

    #[tokio::test]
    async fn missing_bundle_blob_falls_back_to_json() {
        // Arrange
        let chunks = sample_chunks();
        let provider = MockCdn::with_blob_and_json(None, &bundle_json(&chunks));

        // Act
        let loaded = load_bundle_chunks_via(&provider, 0).await.unwrap();

        // Assert
        assert_eq!(loaded, chunks);
        assert!(
            provider
                .requested_paths()
                .contains(&phrase_bundle_json_path(0))
        );
    }

    #[test]
    fn phrase_bundle_blob_guard_is_verified_against_the_source_hash() {
        // Arrange: a blob built with the source-bound guard plus the client
        // expectation derived from the same source hex — the production
        // wiring (guard path = phrases/data_bundle_N.json, VERSIONED_FILES).
        let chunks = sample_chunks();
        let deflated = build_bundle_blob(&chunks, None);
        let expectation = expectation_from_source_json(&chunks);

        // Act: inflate + header + guard, exactly as the client core does.
        let inflated = inflate_blob(&deflated).unwrap();
        let (header, payload) = split_blob(&inflated).unwrap();

        // Assert: the guard matches and the payload parses into the chunks.
        assert!(guard_matches(&header, &expectation));
        let parsed =
            phrase_bundle_chunks_with_expectation(&header, &expectation, payload, 0).unwrap();
        assert_eq!(parsed, chunks);
    }

    #[test]
    fn stale_guard_is_rejected_by_the_injected_expectation() {
        // Arrange: blob bound to a foreign generation, expectation bound to
        // the real source hash (the phrase_loader injectable pattern — the
        // process manifest OnceLock cannot be primed per-case).
        let chunks = sample_chunks();
        let foreign_hashes: Vec<String> = vec![hex_digest(b"older generation")];
        let deflated = build_bundle_blob(&chunks, Some(foreign_hashes));
        let expectation = expectation_from_source_json(&chunks);

        // Act
        let inflated = inflate_blob(&deflated).unwrap();
        let (header, payload) = split_blob(&inflated).unwrap();
        let rejected = phrase_bundle_chunks_with_expectation(&header, &expectation, payload, 0);

        // Assert
        assert!(rejected.is_err(), "a stale guard must be rejected");
    }
}

/// Browser-level behavior of the cleanup/release paths over the real
/// Cache API — the native suite cannot touch web_sys state, and a guard
/// regression in these paths silently reverts the task's own metric
/// (uwuwu #1451 review: the only tests catching it live here).
#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    async fn prime(paths: &[String]) {
        for path in paths {
            cdn_provider::store_text_in_cache(path, "test")
                .await
                .expect("cache store must succeed in the browser runner");
        }
    }

    async fn drop_all(paths: &[String]) {
        for path in paths {
            let _ = cdn_provider::delete_from_cache(path).await;
        }
    }

    #[wasm_bindgen_test]
    async fn release_bundle_cache_entries_removes_both_bundle_copies() {
        // Arrange: both the blob and the legacy JSON bundle are cached.
        let paths = vec![phrase_bundle_blob_path(0), phrase_bundle_json_path(0)];
        drop_all(&paths).await;
        prime(&paths).await;
        for path in &paths {
            assert!(
                cdn_provider::is_cached(path).await,
                "precondition: {path} cached"
            );
        }

        // Act
        release_bundle_cache_entries(0).await;

        // Assert: the bundle copy never survives extraction.
        for path in &paths {
            assert!(
                !cdn_provider::is_cached(path).await,
                "bundle copy must be released: {path}"
            );
        }
    }

    #[wasm_bindgen_test]
    async fn cleanup_removes_legacy_keys_when_replacements_are_cached() {
        // Arrange: every replacement cached, every legacy key present.
        let pairs = legacy_cleanup_pairs();
        let replacements: Vec<String> = pairs.iter().map(|(_, r)| r.clone()).collect();
        let legacy: Vec<String> = pairs.iter().map(|(p, _)| p.clone()).collect();
        drop_all(&replacements).await;
        drop_all(&legacy).await;
        prime(&replacements).await;
        prime(&legacy).await;

        // Act
        cleanup_legacy_json_cache().await;

        // Assert: legacy keys gone, replacements intact.
        for path in &legacy {
            assert!(!cdn_provider::is_cached(path).await, "legacy kept: {path}");
        }
        for path in &replacements {
            assert!(
                cdn_provider::is_cached(path).await,
                "replacement must survive cleanup: {path}"
            );
        }
    }

    #[wasm_bindgen_test]
    async fn cleanup_keeps_legacy_key_whose_replacement_is_missing() {
        // Arrange: legacy keys present, replacements absent — the offline
        // source must survive the pass (review contract).
        let pairs = legacy_cleanup_pairs();
        let replacements: Vec<String> = pairs.iter().map(|(_, r)| r.clone()).collect();
        let legacy: Vec<String> = pairs.iter().map(|(p, _)| p.clone()).collect();
        drop_all(&replacements).await;
        drop_all(&legacy).await;
        prime(&legacy).await;

        // Act
        cleanup_legacy_json_cache().await;

        // Assert: nothing was deleted without a live replacement.
        for path in &legacy {
            assert!(
                cdn_provider::is_cached(path).await,
                "legacy key must be kept without its replacement: {path}"
            );
        }
    }
}
