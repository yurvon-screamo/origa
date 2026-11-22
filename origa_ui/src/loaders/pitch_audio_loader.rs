use origa::dictionary::cdn_blob::{
    DEFLATED_RKYV_SCHEMA_VERSION, BlobFormatError, guard_matches,
};
use origa::dictionary::pitch_audio::{
    ArchivedPitchAudioIndexBlob, access_pitch_index_blob, install_archived_pitch_index,
    is_pitch_audio_loaded,
};
use origa::dictionary::precompute_blob::inflate_and_split;
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cache_manager::guard_expectation_for;
use crate::repository::cdn_provider;
use crate::utils::{now_ms, yield_to_browser};

const PITCH_INDEX_PATH: &str = "pitch/index.json";
const PITCH_INDEX_BLOB_PATH: &str = "pitch/index.rkyv";

/// Why the pitch index blob bytes could not serve the fast path.
#[derive(Debug)]
enum PitchBlobError {
    /// The blob belongs to an older format generation — the cache entry is
    /// stale and heals by purge + refetch (self-heal, uwuwu #1484).
    StaleGeneration,
    /// Corruption, guard mismatch or payload failure — the JSON fallback
    /// serves the load instead.
    Rejected(String),
}

/// Result of the provider-parameterized load: the zero-copy archived view
/// of the pre-parsed CDN blob (fast path) or the raw JSON text (fallback).
/// Installing into the global slot is the wrapper's job, so tests can
/// exercise the core repeatedly.
pub enum LoadedPitchIndex {
    CdnBlob(&'static ArchivedPitchAudioIndexBlob),
    SourceJson(String),
}

pub async fn load_pitch_audio() -> Result<(), OrigaError> {
    if is_pitch_audio_loaded() {
        tracing::debug!("Pitch audio already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("Loading pitch audio index...");

    let loaded = load_pitch_audio_via(cdn_provider()).await?;
    match loaded {
        LoadedPitchIndex::CdnBlob(view) => {
            install_archived_pitch_index(view)?;
            tracing::info!(
                "Pitch audio index loaded: {} entries ({:.2}s total, zero-copy blob)",
                origa::dictionary::pitch_audio::get_audio_entry_count(),
                (now_ms() - start) / 1000.0
            );
        },
        LoadedPitchIndex::SourceJson(json) => {
            yield_to_browser().await;
            let parse_start = now_ms();
            origa::dictionary::pitch_audio::init_pitch_audio_index(&json)?;
            let parse_ms = now_ms() - parse_start;
            tracing::info!(
                "Pitch audio index loaded: {} entries ({:.2}s total, JSON parse {:.2}s)",
                origa::dictionary::pitch_audio::get_audio_entry_count(),
                (now_ms() - start) / 1000.0,
                parse_ms / 1000.0
            );
        },
    }

    Ok(())
}

/// Provider-parameterized core: try the CDN rkyv blob first (zero-copy
/// access — no JSON scan), fall back to the original JSON on any
/// validation or fetch failure. The blob is verified against the remote
/// manifest guard when one was fetched.
///
/// A stale-generation blob heals in place: cache-first would serve it
/// forever, so the loader purges the entry and refetches once — online the
/// refetch is an honest cache miss, offline it fails and the JSON fallback
/// serves the load until the next online start.
pub async fn load_pitch_audio_via<P: CdnProvider>(
    provider: &P,
) -> Result<LoadedPitchIndex, OrigaError> {
    let fetch_start = now_ms();
    let blob_result = provider.fetch_bytes(PITCH_INDEX_BLOB_PATH).await;
    let fetch_ms = now_ms() - fetch_start;

    let access_start = now_ms();
    let view = match blob_result {
        Ok(blob) => match pitch_view_from_blob(&blob) {
            Ok(view) => Some(view),
            Err(PitchBlobError::StaleGeneration) => {
                tracing::warn!(
                    "📖 Pitch index blob of a stale generation, purging cache entry and refetching"
                );
                cdn_provider::purge_stale_blob_entry(PITCH_INDEX_BLOB_PATH).await;
                match provider.fetch_bytes(PITCH_INDEX_BLOB_PATH).await {
                    Ok(refetched) => match pitch_view_from_blob(&refetched) {
                        Ok(view) => Some(view),
                        Err(rejection) => {
                            tracing::warn!(
                                "📖 Pitch index blob rejected after refetch ({rejection:?}), falling back to JSON"
                            );
                            None
                        },
                    },
                    Err(e) => {
                        tracing::warn!(
                            "📖 Pitch index blob refetch unavailable ({e:?}), falling back to JSON"
                        );
                        None
                    },
                }
            },
            Err(rejection) => {
                tracing::warn!("📖 Pitch index rkyv blob rejected ({rejection:?}), falling back to JSON");
                None
            },
        },
        Err(e) => {
            tracing::warn!("📖 Pitch index rkyv blob unavailable ({e:?}), falling back to JSON");
            None
        },
    };

    if let Some(view) = view {
        tracing::debug!(
            "📖 Pitch index blob consumed (fetch {:.2}s, access {:.2}s, zero-copy)",
            fetch_ms / 1000.0,
            (now_ms() - access_start) / 1000.0
        );
        return Ok(LoadedPitchIndex::CdnBlob(view));
    }

    let json = provider.fetch_text(PITCH_INDEX_PATH).await?;
    Ok(LoadedPitchIndex::SourceJson(json))
}

/// Format + guard + payload validation of one pitch index blob: inflate
/// the deflated-rkyv generation, verify the manifest guard, access the
/// payload zero-copy.
fn pitch_view_from_blob(blob: &[u8]) -> Result<&'static ArchivedPitchAudioIndexBlob, PitchBlobError> {
    let (header, payload) = match inflate_and_split(blob, DEFLATED_RKYV_SCHEMA_VERSION) {
        Ok(split) => split,
        Err(BlobFormatError::UnsupportedSchema { .. })
        | Err(BlobFormatError::Inflation { .. }) => {
            // Both stale shapes heal the same way: a cached blob of an
            // older generation does not inflate (the pre-#1484 wire format
            // was raw), and a foreign-generation deflated blob fails the
            // schema check. A blob that fails to inflate at all is also
            // indistinguishable from a poisoned cache body — purging is
            // the established remedy (kanji-art healer).
            return Err(PitchBlobError::StaleGeneration);
        },
        Err(e) => return Err(PitchBlobError::Rejected(e.to_string())),
    };

    let expectation = guard_expectation_for(&[PITCH_INDEX_PATH]);
    if !guard_matches(&header, &expectation) {
        return Err(PitchBlobError::Rejected(
            "stale relative to manifest".to_string(),
        ));
    }

    access_pitch_index_blob(&payload).map_err(|e| PitchBlobError::Rejected(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::future::Future;

    use origa::dictionary::cdn_blob::{
        DEFLATED_RKYV_SCHEMA_VERSION, BlobHeader, SCHEMA_VERSION, build_blob,
        manifest_guard_from_hex_hashes, sha256_bytes,
    };
    use origa::dictionary::pitch_audio::PitchAudioIndex;
    use origa::dictionary::pitch_audio::serialize_pitch_index_blob_to_rkyv;
    use origa::dictionary::precompute_blob::deflate_blob;

    /// Parse helper for the fallback assertions (the owned index is not
    /// installed by the core).
    fn build_pitch_index_from_json(json: &str) -> Result<PitchAudioIndex, OrigaError> {
        PitchAudioIndex::from_json(json)
    }

    const SAMPLE_JSON: &str = r#"{"v":2,"total":1,"entries":{"猫":{"f":"a1b2c3d4.opus","p":1}}}"#;

    /// Recording mock: serves canned bytes per path in request order — an
    /// exhausted queue is the offline-refetch branch of the stale-generation
    /// heal (external CDN boundary — a mock is appropriate).
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
            let result = match self
                .responses
                .borrow()
                .iter()
                .find(|(candidate, _)| candidate == path)
            {
                Some((_, bytes)) => {
                    Ok(String::from_utf8(bytes.clone()).expect("canned text must be UTF-8"))
                },
                None => Err(OrigaError::NetworkError {
                    url: path.to_string(),
                    reason: "not stubbed".to_string(),
                }),
            };
            std::future::ready(result)
        }

        fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
            self.requested.borrow_mut().push(path.to_string());
            let mut responses = self.responses.borrow_mut();
            let position = responses
                .iter()
                .position(|(candidate, _)| candidate == path);
            let result = position
                .map(|index| responses.remove(index).1)
                .ok_or(OrigaError::NetworkError {
                    url: path.to_string(),
                    reason: "not stubbed".to_string(),
                });
            std::future::ready(result)
        }
    }

    /// A deflated-rkyv generation blob (pitch wire format, #1484). The
    /// native tests run without a fetched manifest, so the guard is
    /// `Unavailable` — trusted like every other cache entry offline.
    fn deflated_index_blob(schema: u32, json: &str) -> Vec<u8> {
        let index = build_pitch_index_from_json(json).unwrap();
        let payload = serialize_pitch_index_blob_to_rkyv(&index.to_blob()).unwrap();
        let source_hex = sha256_bytes(json.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let header = BlobHeader {
            schema_version: schema,
            source_sha256: sha256_bytes(json.as_bytes()),
            manifest_guard: manifest_guard_from_hex_hashes(&[&source_hex]),
        };
        deflate_blob(&build_blob(&header, &payload))
    }

    fn valid_blob() -> Vec<u8> {
        deflated_index_blob(DEFLATED_RKYV_SCHEMA_VERSION, SAMPLE_JSON)
    }

    /// Raw-schema-2 blob bytes: what a released client cached before the
    /// deflated generation shipped (the stale-generation heal input).
    fn legacy_raw_blob() -> Vec<u8> {
        let index = build_pitch_index_from_json(SAMPLE_JSON).unwrap();
        let payload = serialize_pitch_index_blob_to_rkyv(&index.to_blob()).unwrap();
        build_blob(
            &BlobHeader {
                schema_version: SCHEMA_VERSION,
                source_sha256: sha256_bytes(SAMPLE_JSON.as_bytes()),
                manifest_guard: manifest_guard_from_hex_hashes(&[]),
            },
            &payload,
        )
    }

    #[tokio::test]
    async fn valid_blob_loads_without_touching_the_json_source() {
        // Arrange
        let provider = MockCdn::serving_blob(PITCH_INDEX_BLOB_PATH, valid_blob());

        // Act
        let loaded = load_pitch_audio_via(&provider).await.unwrap();

        // Assert
        let LoadedPitchIndex::CdnBlob(view) = &loaded else {
            panic!("expected the CDN blob fast path");
        };
        assert_eq!(view.len(), 1);
        assert_eq!(
            view.find_audio_for_reading("猫", "ねこ").unwrap().file(),
            "a1b2c3d4.opus"
        );
        assert!(
            !provider
                .requested_paths()
                .contains(&PITCH_INDEX_PATH.to_string())
        );
    }

    #[tokio::test]
    async fn corrupted_header_falls_back_to_the_json_source() {
        // Arrange: valid payload, destroyed magic
        let mut blob = valid_blob();
        blob[0] = b'X';
        let provider = MockCdn::with_blob_responses(vec![
            (PITCH_INDEX_BLOB_PATH.to_string(), blob),
            (PITCH_INDEX_PATH.to_string(), SAMPLE_JSON.as_bytes().to_vec()),
        ]);

        // Act
        let loaded = load_pitch_audio_via(&provider).await.unwrap();

        // Assert: the fallback JSON must still parse into a usable index.
        let LoadedPitchIndex::SourceJson(json) = loaded else {
            panic!("expected the JSON fallback");
        };
        let index = build_pitch_index_from_json(&json).unwrap();
        assert!(index.get_entry("猫").is_some());
    }

    #[tokio::test]
    async fn missing_blob_falls_back_to_the_json_source() {
        // Arrange: no blob canned — the JSON source serves the load.
        let provider = MockCdn::with_blob_responses(vec![(
            PITCH_INDEX_PATH.to_string(),
            SAMPLE_JSON.as_bytes().to_vec(),
        )]);

        // Act
        let loaded = load_pitch_audio_via(&provider).await.unwrap();

        // Assert
        assert!(matches!(loaded, LoadedPitchIndex::SourceJson(_)));
        assert!(
            provider
                .requested_paths()
                .contains(&PITCH_INDEX_BLOB_PATH.to_string())
        );
    }

    #[tokio::test]
    async fn stale_generation_blob_purges_and_heals_from_the_refetch() {
        // Arrange: a released client cached the pre-#1484 raw blob; the
        // refetch (post-purge cache miss) serves the new generation. The
        // cache purge itself is a wasm Cache-API primitive — best-effort,
        // exercised by the wasm suite.
        let provider = MockCdn::with_blob_responses(vec![
            (PITCH_INDEX_BLOB_PATH.to_string(), legacy_raw_blob()),
            (PITCH_INDEX_BLOB_PATH.to_string(), valid_blob()),
        ]);

        // Act
        let loaded = load_pitch_audio_via(&provider).await.unwrap();

        // Assert: the heal consumed both canned responses and the fresh
        // generation served the load.
        assert!(
            matches!(loaded, LoadedPitchIndex::CdnBlob(_)),
            "refetch after purge must heal the load"
        );
        assert_eq!(provider.requested_paths().len(), 2);
        assert!(
            !provider
                .requested_paths()
                .contains(&PITCH_INDEX_PATH.to_string())
        );
    }

    #[tokio::test]
    async fn stale_generation_blob_falls_back_to_json_when_the_refetch_fails() {
        // Arrange: the stale blob is the only canned response — the refetch
        // fails (offline) and the JSON source serves the load.
        let provider = MockCdn::with_blob_responses(vec![
            (PITCH_INDEX_BLOB_PATH.to_string(), legacy_raw_blob()),
            (PITCH_INDEX_PATH.to_string(), SAMPLE_JSON.as_bytes().to_vec()),
        ]);

        // Act
        let loaded = load_pitch_audio_via(&provider).await.unwrap();

        // Assert
        assert!(matches!(loaded, LoadedPitchIndex::SourceJson(_)));
        assert_eq!(provider.requested_paths().len(), 3, "blob, refetch, json");
    }
}
