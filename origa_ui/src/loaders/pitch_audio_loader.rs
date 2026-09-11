use origa::dictionary::cdn_blob::{guard_matches, split_blob};
use origa::dictionary::pitch_audio::{
    ArchivedPitchAudioIndexBlob, access_pitch_index_blob, install_archived_pitch_index,
    is_pitch_audio_loaded,
};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cache_manager::guard_expectation_for;
use crate::repository::cdn_provider;
use crate::utils::{now_ms, yield_to_browser};

const PITCH_INDEX_PATH: &str = "pitch/index.json";
const PITCH_INDEX_BLOB_PATH: &str = "pitch/index.rkyv";

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
pub async fn load_pitch_audio_via<P: CdnProvider>(
    provider: &P,
) -> Result<LoadedPitchIndex, OrigaError> {
    let fetch_start = now_ms();
    match provider.fetch_bytes(PITCH_INDEX_BLOB_PATH).await {
        Ok(blob) => {
            let fetch_ms = now_ms() - fetch_start;
            let access_start = now_ms();
            match pitch_view_from_blob(&blob) {
                Ok(view) => {
                    tracing::debug!(
                        "📖 Pitch index blob consumed (fetch {:.2}s, access {:.2}s, zero-copy)",
                        fetch_ms / 1000.0,
                        (now_ms() - access_start) / 1000.0
                    );
                    return Ok(LoadedPitchIndex::CdnBlob(view));
                },
                Err(e) => {
                    tracing::warn!(
                        "📖 Pitch index rkyv blob rejected ({e:?}), falling back to JSON"
                    );
                },
            }
        },
        Err(e) => {
            tracing::warn!("📖 Pitch index rkyv blob unavailable ({e:?}), falling back to JSON");
        },
    }

    let json = provider.fetch_text(PITCH_INDEX_PATH).await?;
    Ok(LoadedPitchIndex::SourceJson(json))
}

/// Validate the blob header, verify the manifest guard and access the
/// payload as a zero-copy view. Any failure yields `Err` (caller falls
/// back to the JSON path).
fn pitch_view_from_blob(blob: &[u8]) -> Result<&'static ArchivedPitchAudioIndexBlob, OrigaError> {
    let (header, payload) = split_blob(blob).map_err(|e| OrigaError::PitchAudioParseError {
        reason: format!("pitch index blob header invalid: {e}"),
    })?;

    let expectation = guard_expectation_for(&[PITCH_INDEX_PATH]);
    if !guard_matches(&header, &expectation) {
        return Err(OrigaError::PitchAudioParseError {
            reason: "pitch index blob stale relative to manifest".to_string(),
        });
    }

    access_pitch_index_blob(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::future::Future;

    use origa::dictionary::cdn_blob::{
        BlobHeader, SCHEMA_VERSION, build_blob, manifest_guard_from_hex_hashes, sha256_bytes,
    };
    use origa::dictionary::pitch_audio::PitchAudioIndex;
    use origa::dictionary::pitch_audio::serialize_pitch_index_blob_to_rkyv;

    /// Parse helper for the fallback assertions (the owned index is not
    /// installed by the core).
    fn build_pitch_index_from_json(json: &str) -> Result<PitchAudioIndex, OrigaError> {
        PitchAudioIndex::from_json(json)
    }

    const SAMPLE_JSON: &str = r#"{"v":2,"total":1,"entries":{"猫":{"f":"a1b2c3d4.opus","p":1}}}"#;

    /// Recording mock: serves canned bytes/text per path and tracks which
    /// paths were requested (external CDN boundary — a mock is appropriate).
    struct MockCdn {
        responses: RefCell<Vec<(String, MockResponse)>>,
        requested: RefCell<Vec<String>>,
    }

    enum MockResponse {
        Bytes(Vec<u8>),
        Text(String),
    }

    impl MockCdn {
        fn with_blob_and_json(blob: Option<Vec<u8>>, json: &str) -> Self {
            let mut responses = Vec::new();
            if let Some(bytes) = blob {
                responses.push((
                    PITCH_INDEX_BLOB_PATH.to_string(),
                    MockResponse::Bytes(bytes),
                ));
            }
            responses.push((
                PITCH_INDEX_PATH.to_string(),
                MockResponse::Text(json.to_string()),
            ));
            Self {
                responses: RefCell::new(responses),
                requested: RefCell::new(Vec::new()),
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

    fn valid_blob(json: &str) -> Vec<u8> {
        let index = build_pitch_index_from_json(json).unwrap();
        let payload = serialize_pitch_index_blob_to_rkyv(&index.to_blob()).unwrap();
        let source_hex = sha256_bytes(json.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let header = BlobHeader {
            schema_version: SCHEMA_VERSION,
            source_sha256: sha256_bytes(json.as_bytes()),
            manifest_guard: manifest_guard_from_hex_hashes(&[&source_hex]),
        };
        build_blob(&header, &payload)
    }

    #[tokio::test]
    async fn valid_blob_loads_without_touching_the_json_source() {
        // Arrange
        let provider = MockCdn::with_blob_and_json(Some(valid_blob(SAMPLE_JSON)), SAMPLE_JSON);

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
        let mut blob = valid_blob(SAMPLE_JSON);
        blob[0] = b'X';
        let provider = MockCdn::with_blob_and_json(Some(blob), SAMPLE_JSON);

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
        // Arrange
        let provider = MockCdn::with_blob_and_json(None, SAMPLE_JSON);

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
    async fn future_schema_version_falls_back_to_the_json_source() {
        // Arrange
        let mut blob = valid_blob(SAMPLE_JSON);
        let version = SCHEMA_VERSION + 1;
        blob[4..8].copy_from_slice(&version.to_le_bytes());
        let provider = MockCdn::with_blob_and_json(Some(blob), SAMPLE_JSON);

        // Act
        let loaded = load_pitch_audio_via(&provider).await.unwrap();

        // Assert
        assert!(matches!(loaded, LoadedPitchIndex::SourceJson(_)));
    }
}
