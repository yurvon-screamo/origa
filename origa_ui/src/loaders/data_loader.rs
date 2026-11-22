use origa::dictionary::cdn_blob::{
    BlobFormatError, DEFLATED_RKYV_SCHEMA_VERSION, SCHEMA_VERSION, guard_matches,
};
use origa::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
use origa::dictionary::kanji::{KanjiData, init_kanji, is_kanji_loaded};
use origa::dictionary::precompute_blob::inflate_and_split;
use origa::dictionary::radical::{RadicalData, init_radicals, is_radicals_loaded};
use origa::dictionary::vocabulary::{
    ArchivedVocabularyBlob, VocabularyChunkData, access_vocabulary_blob, init_vocabulary,
    init_vocabulary_from_rkyv, install_archived_vocabulary, is_vocabulary_loaded,
    serialize_vocabulary_to_rkyv,
};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cache_manager::guard_expectation_for;
use crate::repository::cdn_provider;
use crate::repository::cdn_provider::BlobLoadRejection;
use crate::repository::{get_cached_vocabulary_rkyv, save_vocabulary_to_cache_rkyv};
use crate::utils::{now_ms, yield_to_browser};

/// CDN path of the pre-built deterministic vocabulary blob.
const VOCABULARY_BLOB_PATH: &str = "dictionary/vocabulary.rkyv";

/// Chunk paths feeding the vocabulary blob — used for the manifest guard
/// expectation and the precache legacy cleanup pairs.
pub(crate) const VOCABULARY_CHUNK_PATHS: [&str; 11] = [
    "dictionary/chunk_01.json",
    "dictionary/chunk_02.json",
    "dictionary/chunk_03.json",
    "dictionary/chunk_04.json",
    "dictionary/chunk_05.json",
    "dictionary/chunk_06.json",
    "dictionary/chunk_07.json",
    "dictionary/chunk_08.json",
    "dictionary/chunk_09.json",
    "dictionary/chunk_10.json",
    "dictionary/chunk_11.json",
];

pub async fn load_vocabulary() -> Result<(), OrigaError> {
    if is_vocabulary_loaded() {
        tracing::debug!("📖 Vocabulary already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading vocabulary...");

    // Fastest path: pre-built rkyv blob from the CDN (cache-first),
    // consumed via zero-copy access — no owned-structure build and no
    // client-side re-serialization into the local rkyv cache.
    if let Some(view) = load_vocabulary_from_cdn_blob(cdn_provider()).await? {
        install_archived_vocabulary(view)?;
        tracing::info!(
            "📖 Vocabulary loaded from CDN rkyv blob ({:.2}s)",
            (now_ms() - start) / 1000.0
        );
        return Ok(());
    }

    // Known coverage gap (accepted at review): the fallback chain below —
    // client rkyv cache → JSON chunks → client re-serialization — lives in
    // this wrapper because the Cache API layer is not natively testable; it
    // is behaviour-covered by e2e only. The CDN-rkyv core above carries the
    // native mock tests.
    // Fast path: try loading from the rkyv cache — the deterministic CDN
    // blob form, consumed via zero-copy access (see
    // `init_vocabulary_from_rkyv`). A pre-unification cache entry (the old
    // VocabularyDatabase form) fails access and falls through to network.
    match get_cached_vocabulary_rkyv().await {
        Ok(Some(bytes)) => {
            tracing::info!("📖 Cached vocabulary found, {} bytes", bytes.len());
            yield_to_browser().await;
            match init_vocabulary_from_rkyv(&bytes) {
                Ok(()) => {
                    tracing::info!(
                        "📖 Vocabulary loaded from rkyv cache ({:.2}s)",
                        (now_ms() - start) / 1000.0
                    );
                    return Ok(());
                },
                Err(e) => {
                    tracing::warn!(
                        error = ?e,
                        "Failed to load vocabulary from rkyv cache, falling back to network"
                    );
                },
            }
        },
        Ok(None) => {
            tracing::debug!("📖 No rkyv vocabulary cache found, loading from network");
        },
        Err(e) => {
            tracing::warn!(
                "Vocabulary cache read failed, loading from network: {:?}",
                e
            );
        },
    }

    // Slow path: fetch JSON chunks from CDN and parse.
    let cdn = cdn_provider();

    // Batched fetch: 11 JSON chunks (~35 MB total). Fetching all 11 at once
    // via join_all caused WASM OOM on iOS. Batches of 3 keep peak memory
    // bounded while still parallelizing for speed (~4 rounds instead of 11
    // sequential requests).
    const BATCH_SIZE: usize = 3;
    let mut all_chunks: Vec<String> = Vec::with_capacity(11);

    for batch_start in (1..=11).step_by(BATCH_SIZE) {
        let batch_end = (batch_start + BATCH_SIZE - 1).min(11);
        let batch_futures: Vec<_> = (batch_start..=batch_end)
            .map(|i| {
                let path = format!("dictionary/chunk_{:02}.json", i);
                async move { cdn.fetch_text(&path).await }
            })
            .collect();
        let batch_results = futures::future::join_all(batch_futures).await;
        for result in batch_results {
            all_chunks.push(result?);
        }
        yield_to_browser().await;
    }

    let data = VocabularyChunkData {
        chunk_01: all_chunks[0].clone(),
        chunk_02: all_chunks[1].clone(),
        chunk_03: all_chunks[2].clone(),
        chunk_04: all_chunks[3].clone(),
        chunk_05: all_chunks[4].clone(),
        chunk_06: all_chunks[5].clone(),
        chunk_07: all_chunks[6].clone(),
        chunk_08: all_chunks[7].clone(),
        chunk_09: all_chunks[8].clone(),
        chunk_10: all_chunks[9].clone(),
        chunk_11: all_chunks[10].clone(),
    };

    yield_to_browser().await;
    init_vocabulary(data)?;

    // Cache the deterministic CDN-blob form for future fast-path loads.
    let cache_start = now_ms();
    match serialize_vocabulary_to_rkyv() {
        Ok(mut bytes) => {
            if let Err(e) = save_vocabulary_to_cache_rkyv(&mut bytes).await {
                tracing::warn!("Failed to cache vocabulary (rkyv): {:?}", e);
            } else {
                tracing::info!(
                    "📖 Vocabulary cached (rkyv, {} bytes, {:.2}s)",
                    bytes.len(),
                    (now_ms() - cache_start) / 1000.0
                );
            }
        },
        Err(e) => {
            tracing::warn!("Failed to serialize vocabulary for caching: {:?}", e);
        },
    }

    tracing::info!("📖 Vocabulary loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

/// Format + guard + payload validation of one vocabulary blob: inflate the
/// deflated-rkyv generation, verify the manifest guard over the chunk
/// hashes, access the payload zero-copy. Stale generations — and bytes
/// that do not inflate at all, which is what the pre-#1484 raw wire
/// format looks like to a deflated reader — surface as `StaleGeneration`
/// so the shared read path heals them by purge + refetch.
fn vocabulary_view_from_blob(
    blob: &[u8],
) -> Result<&'static ArchivedVocabularyBlob, BlobLoadRejection> {
    let (header, payload) = match inflate_and_split(blob, DEFLATED_RKYV_SCHEMA_VERSION) {
        Ok(split) => split,
        Err(BlobFormatError::UnsupportedSchema { .. }) | Err(BlobFormatError::Inflation { .. }) => {
            return Err(BlobLoadRejection::StaleGeneration);
        },
        Err(e) => return Err(BlobLoadRejection::Rejected(e.to_string())),
    };
    let expectation = guard_expectation_for(&VOCABULARY_CHUNK_PATHS);
    if !guard_matches(&header, &expectation) {
        return Err(BlobLoadRejection::Rejected(
            "stale relative to manifest".to_string(),
        ));
    }
    access_vocabulary_blob(&payload).map_err(|e| BlobLoadRejection::Rejected(e.to_string()))
}

/// CDN-rkyv fast path core. `Ok(None)` means "no usable blob — use the
/// fallback chain" (fetch error, invalid header, guard mismatch or payload
/// failure are all non-fatal: the original sources remain loadable). The
/// returned archived view is installed by the caller — tests exercise this
/// core without touching the one-shot global slot.
async fn load_vocabulary_from_cdn_blob<P: CdnProvider>(
    provider: &P,
) -> Result<Option<&'static ArchivedVocabularyBlob>, OrigaError> {
    cdn_provider::load_deflated_rkyv_blob(
        provider,
        VOCABULARY_BLOB_PATH,
        "Vocabulary",
        vocabulary_view_from_blob,
    )
    .await
}

/// Deflated-JSON blob fast path shared by grammar/kanji/radicals (uwuwu
/// #1484): fetch → inflate → header guard over the JSON source manifest
/// hash → payload as UTF-8. These paths never carried another format, so
/// there is no stale-generation heal — `Ok(None)` means "no usable blob,
/// use the verified text fallback" (fetch error, guard mismatch, invalid
/// header — all non-fatal).
async fn load_deflated_json_via<P: CdnProvider>(
    provider: &P,
    blob_path: &str,
    source_path: &str,
    resource: &str,
) -> Result<Option<String>, OrigaError> {
    let deflated = match provider.fetch_bytes(blob_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::warn!("📖 {resource} blob unavailable ({e:?}), using JSON fallback");
            return Ok(None);
        },
    };
    let (header, payload) = match inflate_and_split(&deflated, SCHEMA_VERSION) {
        Ok(split) => split,
        Err(e) => {
            tracing::warn!("📖 {resource} blob rejected ({e}), using JSON fallback");
            return Ok(None);
        },
    };
    let expectation = guard_expectation_for(&[source_path]);
    if !guard_matches(&header, &expectation) {
        tracing::warn!("📖 {resource} blob stale relative to manifest, using JSON fallback");
        return Ok(None);
    }
    match String::from_utf8(payload) {
        Ok(json) => Ok(Some(json)),
        Err(e) => {
            tracing::warn!("📖 {resource} blob payload is not UTF-8 ({e}), using JSON fallback");
            Ok(None)
        },
    }
}

pub async fn load_kanji() -> Result<(), OrigaError> {
    if is_kanji_loaded() {
        tracing::debug!("📖 Kanji already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading kanji...");

    // Fast path: the deflated-JSON blob (uwuwu #1484); its guard recompute
    // off the manifest replaces the sha256 body pass of the verified fetch.
    let json = match load_deflated_json_via(
        cdn_provider(),
        "dictionary/kanji.rkyv",
        "dictionary/kanji.json",
        "kanji",
    )
    .await?
    {
        Some(json) => json,
        None => {
            // Verified fetch: a cache body predating the file's manifest
            // tracking never invalidates through the manifest diff — see
            // fetch_text_verified.
            cdn_provider::fetch_text_verified("dictionary/kanji.json").await?
        },
    };
    let data = KanjiData { kanji_json: json };

    yield_to_browser().await;
    init_kanji(data)?;

    tracing::info!("📖 Kanji loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

pub async fn load_grammar() -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        tracing::debug!("📖 Grammar already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading grammar...");

    // v3 corpus (#503 content pass): 677 rules, all four locales in one
    // file — the only grammar format on the CDN since the legacy
    // retirement (#583/#507/#482 removed v1, v2 and the KO/VI overlay).
    // Fast path: the deflated-JSON blob (uwuwu #1484); its guard recompute
    // off the manifest replaces the sha256 body pass of the verified fetch.
    let json = match load_deflated_json_via(
        cdn_provider(),
        "grammar/grammar_v3.rkyv",
        "grammar/grammar_v3.json",
        "grammar",
    )
    .await?
    {
        Some(json) => json,
        None => {
            // Verified fetch: a cache body predating the file's manifest
            // tracking never invalidates through the manifest diff — see
            // fetch_text_verified.
            cdn_provider::fetch_text_verified("grammar/grammar_v3.json").await?
        },
    };
    let data = GrammarData { grammar_json: json };

    yield_to_browser().await;
    init_grammar(data)?;

    // Best-effort precompute install (#521): grammar markdown nodes and
    // whole strings were keyed offline. A missing/stale blob only means
    // the live tokenizer path keeps serving grammar renders. Spawned
    // (NOT awaited): the 4.2 MB blob must never hold the grammar
    // readiness flag — and with it the startup overlay — hostage.
    leptos::task::spawn_local(async move {
        if let Err(e) = crate::loaders::grammar_precompute_loader::load_grammar_precompute().await {
            tracing::warn!("📖 Grammar precompute unavailable ({e:?}), live path serves renders");
        }
    });

    tracing::info!("📖 Grammar loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

pub async fn load_counters() -> Result<(), OrigaError> {
    if origa::dictionary::counters::is_counters_loaded() {
        tracing::debug!("🔢 Counters already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("🔢 Loading counters...");

    // Verified fetch: a cache body predating the file's manifest tracking
    // never invalidates through the manifest diff — see fetch_text_verified.
    let json = cdn_provider::fetch_text_verified("counters/counters.json").await?;

    yield_to_browser().await;
    origa::dictionary::counters::init_counters(&json).map_err(|e| OrigaError::RepositoryError {
        reason: format!("counters registry: {e}"),
    })?;

    tracing::info!("🔢 Counters loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

pub async fn load_radicals() -> Result<(), OrigaError> {
    if is_radicals_loaded() {
        tracing::debug!("📖 Radicals already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading radicals...");

    // Fast path: the deflated-JSON blob (uwuwu #1484); its guard recompute
    // off the manifest replaces the sha256 body pass of the verified fetch.
    let json = match load_deflated_json_via(
        cdn_provider(),
        "dictionary/radicals.rkyv",
        "dictionary/radicals.json",
        "radicals",
    )
    .await?
    {
        Some(json) => json,
        None => {
            // Verified fetch: a cache body predating the file's manifest
            // tracking never invalidates through the manifest diff — see
            // fetch_text_verified.
            cdn_provider::fetch_text_verified("dictionary/radicals.json").await?
        },
    };
    let data = RadicalData {
        radicals_json: json,
    };

    yield_to_browser().await;
    init_radicals(data)?;

    tracing::info!("📖 Radicals loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::future::Future;

    use origa::dictionary::cdn_blob::{
        BlobHeader, DEFLATED_RKYV_SCHEMA_VERSION, SCHEMA_VERSION, build_blob,
        manifest_guard_from_hex_hashes, sha256_bytes,
    };
    use origa::dictionary::precompute_blob::deflate_blob;
    use origa::dictionary::vocabulary::{
        build_vocabulary_database_from_chunks, serialize_vocabulary_blob_to_rkyv,
    };

    use super::*;

    fn chunk_json() -> String {
        r#"{"猫": {"russian_translation": "кот", "english_translation": "cat"}}"#.to_string()
    }

    fn chunk_data() -> VocabularyChunkData {
        VocabularyChunkData {
            chunk_01: chunk_json(),
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

    /// Recording mock over the external CDN boundary: (path, bytes) pairs
    /// consumed in request order — a request with no remaining canned
    /// response for its path fails as "not stubbed" (an exhausted queue is
    /// the offline-refetch branch of the stale-generation heal).
    struct MockCdn {
        responses: RefCell<Vec<(String, Vec<u8>)>>,
        requested: RefCell<Vec<String>>,
    }

    impl MockCdn {
        fn with_blob_responses(responses: Vec<(String, Vec<u8>)>) -> Self {
            Self {
                responses: RefCell::new(responses),
                requested: RefCell::new(Vec::new()),
            }
        }

        fn serving_blob(path: &str, bytes: Vec<u8>) -> Self {
            Self::with_blob_responses(vec![(path.to_string(), bytes)])
        }

        fn requested_paths(&self) -> Vec<String> {
            self.requested.borrow().clone()
        }
    }

    impl CdnProvider for MockCdn {
        fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
            self.requested.borrow_mut().push(path.to_string());
            std::future::ready(Err(OrigaError::NetworkError {
                url: path.to_string(),
                reason: "text fetch is not stubbed in blob tests".to_string(),
            }))
        }

        fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
            self.requested.borrow_mut().push(path.to_string());
            let mut responses = self.responses.borrow_mut();
            let position = responses
                .iter()
                .position(|(candidate, _)| candidate == path);
            let result =
                position
                    .map(|index| responses.remove(index).1)
                    .ok_or(OrigaError::NetworkError {
                        url: path.to_string(),
                        reason: "not stubbed".to_string(),
                    });
            std::future::ready(result)
        }
    }

    /// A deflated-rkyv generation blob (vocabulary wire format, #1484).
    fn wrap_deflated_rkyv(schema: u32, payload: Vec<u8>, guard: [u8; 32]) -> Vec<u8> {
        let header = BlobHeader {
            schema_version: schema,
            source_sha256: sha256_bytes(b"concatenated chunks"),
            manifest_guard: guard,
        };
        deflate_blob(&build_blob(&header, &payload))
    }

    fn valid_blob() -> Vec<u8> {
        let database = build_vocabulary_database_from_chunks(chunk_data()).unwrap();
        let payload = serialize_vocabulary_blob_to_rkyv(&database).unwrap();

        let chunk_bytes = chunk_json().into_bytes();
        let hex_hashes: Vec<String> = std::iter::repeat_n(
            {
                let digest = sha256_bytes(&chunk_bytes);
                digest
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            },
            VOCABULARY_CHUNK_PATHS.len(),
        )
        .collect();
        let hex_refs: Vec<&str> = hex_hashes.iter().map(String::as_str).collect();

        wrap_deflated_rkyv(
            DEFLATED_RKYV_SCHEMA_VERSION,
            payload,
            manifest_guard_from_hex_hashes(&hex_refs),
        )
    }

    /// Raw-schema-2 blob bytes: what a released client cached before the
    /// deflated generation shipped (the stale-generation heal input).
    fn legacy_raw_blob() -> Vec<u8> {
        let payload = serialize_vocabulary_blob_to_rkyv(
            &build_vocabulary_database_from_chunks(chunk_data()).unwrap(),
        )
        .unwrap();
        build_blob(
            &BlobHeader {
                schema_version: SCHEMA_VERSION,
                source_sha256: sha256_bytes(b"concatenated chunks"),
                manifest_guard: manifest_guard_from_hex_hashes(&[]),
            },
            &payload,
        )
    }

    /// A deflated-JSON blob (grammar/kanji/radicals wire format, #1484).
    /// The native tests run without a fetched manifest, so the guard is
    /// `Unavailable` — trusted like every other cache entry offline.
    fn deflated_json_blob(json: &str) -> Vec<u8> {
        let hex = {
            let digest = sha256_bytes(json.as_bytes());
            digest
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        let header = BlobHeader {
            schema_version: SCHEMA_VERSION,
            source_sha256: sha256_bytes(json.as_bytes()),
            manifest_guard: manifest_guard_from_hex_hashes(&[&hex]),
        };
        deflate_blob(&build_blob(&header, json.as_bytes()))
    }

    #[tokio::test]
    async fn valid_blob_loads_database_without_fetching_chunks() {
        // Arrange
        let provider = MockCdn::serving_blob(VOCABULARY_BLOB_PATH, valid_blob());

        // Act
        let view = load_vocabulary_from_cdn_blob(&provider)
            .await
            .unwrap()
            .expect("valid blob must load");

        // Assert
        assert_eq!(
            view.translations("猫", &origa::domain::NativeLanguage::Russian),
            Some(vec!["кот".to_string()])
        );
        let chunk_requested = provider
            .requested_paths()
            .iter()
            .any(|path| path.starts_with("dictionary/chunk_"));
        assert!(!chunk_requested, "fast path must not fetch JSON chunks");
    }

    #[tokio::test]
    async fn missing_blob_yields_none_for_the_fallback_chain() {
        // Arrange
        let provider = MockCdn::with_blob_responses(Vec::new());

        // Act
        let view = load_vocabulary_from_cdn_blob(&provider).await.unwrap();

        // Assert
        assert!(view.is_none());
    }

    #[tokio::test]
    async fn corrupted_header_yields_none_for_the_fallback_chain() {
        // Arrange
        let mut blob = valid_blob();
        blob[0] = b'X';
        let provider = MockCdn::serving_blob(VOCABULARY_BLOB_PATH, blob);

        // Act
        let view = load_vocabulary_from_cdn_blob(&provider).await.unwrap();

        // Assert
        assert!(view.is_none());
    }

    #[tokio::test]
    async fn stale_generation_blob_purges_and_heals_from_the_refetch() {
        // Arrange: a released client cached the pre-#1484 raw blob; the
        // refetch (post-purge cache miss) serves the new generation. The
        // cache purge itself is a wasm Cache-API primitive — best-effort,
        // exercised by the wasm suite.
        let provider = MockCdn::with_blob_responses(vec![
            (VOCABULARY_BLOB_PATH.to_string(), legacy_raw_blob()),
            (VOCABULARY_BLOB_PATH.to_string(), valid_blob()),
        ]);

        // Act
        let view = load_vocabulary_from_cdn_blob(&provider).await.unwrap();

        // Assert: the heal consumed both canned responses and the fresh
        // generation served the load — no fallback-chain signals.
        assert!(view.is_some(), "refetch after purge must heal the load");
        assert_eq!(provider.requested_paths().len(), 2);
    }

    #[tokio::test]
    async fn stale_generation_blob_yields_none_when_the_refetch_fails() {
        // Arrange: the stale blob is the only canned response — the refetch
        // fails (offline) and the fallback chain serves the load.
        let provider = MockCdn::serving_blob(VOCABULARY_BLOB_PATH, legacy_raw_blob());

        // Act
        let view = load_vocabulary_from_cdn_blob(&provider).await.unwrap();

        // Assert
        assert!(view.is_none());
        assert_eq!(provider.requested_paths().len(), 2, "refetch happened");
    }

    #[tokio::test]
    async fn deflated_json_blob_serves_the_payload_as_text() {
        // Arrange
        let json = r#"{"rules":[{"id":"g1"}]}"#;
        let provider = MockCdn::serving_blob("grammar/grammar_v3.rkyv", deflated_json_blob(json));

        // Act
        let loaded = load_deflated_json_via(
            &provider,
            "grammar/grammar_v3.rkyv",
            "grammar/grammar_v3.json",
            "grammar",
        )
        .await
        .unwrap();

        // Assert
        assert_eq!(loaded.as_deref(), Some(json));
    }

    #[tokio::test]
    async fn rejected_deflated_json_blob_yields_none_for_the_verified_fallback() {
        // Arrange: bytes that do not inflate
        let provider = MockCdn::serving_blob("dictionary/kanji.rkyv", b"garbage".to_vec());

        // Act
        let loaded = load_deflated_json_via(
            &provider,
            "dictionary/kanji.rkyv",
            "dictionary/kanji.json",
            "kanji",
        )
        .await
        .unwrap();

        // Assert
        assert!(loaded.is_none());
    }

    /// Regression pin for the verified-fetch migration (task
    /// 20261107150931-k0c1rb7): `load_kanji`/`load_grammar`/`load_radicals`
    /// must keep their JSON fallback on `fetch_text_verified`. The loaders
    /// have no injection seam (global provider + OnceLock init), so the
    /// behavioral coverage of the #1484 blob fast paths lives in the
    /// MockCdn tests above and in `dictionary_verified_fetch_wasm_tests`;
    /// this pin catches a revert to plain `fetch_text` at the source level.
    /// Update the expectations together with the call sites.
    #[test]
    fn dictionary_loaders_fetch_through_verified_gate() {
        let source = include_str!("data_loader.rs");

        for path in [
            "dictionary/kanji.json",
            "grammar/grammar_v3.json",
            "dictionary/radicals.json",
        ] {
            let verified = format!("fetch_text_verified(\"{path}\")");
            assert!(
                source.contains(&verified),
                "{path} must be fetched through fetch_text_verified"
            );
            let plain = format!("fetch_text(\"{path}\")");
            assert!(
                !source.contains(&plain),
                "{path} must not be fetched through plain fetch_text"
            );
        }
    }
}
