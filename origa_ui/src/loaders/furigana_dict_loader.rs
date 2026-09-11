use origa::dictionary::cdn_blob::{guard_matches, split_blob};
use origa::dictionary::furigana_dict::{
    ArchivedFuriganaDictionary, FuriganaDictionary, access_furigana_payload,
    build_furigana_dict_from_text, install_archived_furigana, is_furigana_dict_loaded,
    set_furigana_dict,
};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cache_manager::guard_expectation_for;
use crate::repository::cdn_provider;
use crate::utils::now_ms;

const FURIGANA_DICT_PATH: &str = "dictionaries/JmdictFurigana.txt";
const FURIGANA_BLOB_PATH: &str = "dictionaries/JmdictFurigana.rkyv";

/// Result of the provider-parameterized load: either the zero-copy archived
/// view of the pre-parsed CDN blob (fast path) or the dictionary built from
/// the original JmdictFurigana text (fallback). Installing into the global
/// slot is the wrapper's job, so tests can exercise the core repeatedly.
pub enum LoadedFurigana {
    CdnBlob(&'static ArchivedFuriganaDictionary),
    SourceText(FuriganaDictionary),
}

pub async fn load_furigana_dict() -> Result<(), OrigaError> {
    if is_furigana_dict_loaded() {
        tracing::debug!("📖 Furigana dictionary already loaded");
        return Ok(());
    }

    let start = now_ms();
    let loaded = load_furigana_dict_via(cdn_provider()).await?;
    match loaded {
        LoadedFurigana::CdnBlob(view) => install_archived_furigana(view)?,
        LoadedFurigana::SourceText(dict) => set_furigana_dict(dict)?,
    }

    tracing::info!(
        "📖 Furigana dictionary loaded ({:.2}s)",
        (now_ms() - start) / 1000.0
    );
    Ok(())
}

/// Provider-parameterized core: try the CDN rkyv blob first (zero-copy
/// access, no owned-structure build), fall back to the original text
/// source on any validation or fetch failure. The blob is verified against
/// the remote manifest guard when one was fetched (offline starts trust
/// the cache, matching every other cache entry).
pub async fn load_furigana_dict_via<P: CdnProvider>(
    provider: &P,
) -> Result<LoadedFurigana, OrigaError> {
    let fetch_start = now_ms();
    match provider.fetch_bytes(FURIGANA_BLOB_PATH).await {
        Ok(blob) => {
            let fetch_ms = now_ms() - fetch_start;
            if let Some(view) = furigana_view_from_blob(&blob) {
                tracing::debug!(
                    "📖 Furigana blob consumed (fetch {:.2}s, zero-copy)",
                    fetch_ms / 1000.0
                );
                return Ok(LoadedFurigana::CdnBlob(view));
            }
            tracing::warn!("📖 Furigana rkyv blob rejected, falling back to text source");
        },
        Err(e) => {
            tracing::warn!(
                "📖 Furigana rkyv blob unavailable ({e:?}), falling back to text source"
            );
        },
    }

    let text = provider.fetch_text(FURIGANA_DICT_PATH).await?;
    Ok(LoadedFurigana::SourceText(build_furigana_dict_from_text(
        &text,
    )?))
}

/// Validate the blob header, verify the manifest guard and access the
/// payload as a zero-copy view. Any failure yields `None` (caller falls
/// back to the text path).
fn furigana_view_from_blob(blob: &[u8]) -> Option<&'static ArchivedFuriganaDictionary> {
    let (header, payload) = match split_blob(blob) {
        Ok(split) => split,
        Err(e) => {
            tracing::warn!("📖 Furigana blob header invalid: {e}");
            return None;
        },
    };

    let expectation = guard_expectation_for(&[FURIGANA_DICT_PATH]);
    if !guard_matches(&header, &expectation) {
        tracing::warn!("📖 Furigana blob stale relative to manifest, falling back");
        return None;
    }

    match access_furigana_payload(payload) {
        Ok(view) => Some(view),
        Err(e) => {
            tracing::warn!("📖 Furigana blob payload failed to access: {e:?}");
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::future::Future;

    use origa::dictionary::cdn_blob::{
        BlobHeader, SCHEMA_VERSION, build_blob, manifest_guard_from_hex_hashes, sha256_bytes,
    };
    use origa::dictionary::furigana_dict::serialize_furigana_dict_to_rkyv;

    use super::*;

    const SAMPLE_TEXT: &str = "指|ゆび|0:ゆび\n大人|おとな|0-1:おとな";

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
        fn with_blob_and_text(blob: Option<Vec<u8>>, text: &str) -> Self {
            let mut responses = Vec::new();
            if let Some(bytes) = blob {
                responses.push((FURIGANA_BLOB_PATH.to_string(), MockResponse::Bytes(bytes)));
            }
            responses.push((
                FURIGANA_DICT_PATH.to_string(),
                MockResponse::Text(text.to_string()),
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

    fn valid_blob(text: &str) -> Vec<u8> {
        let dict = build_furigana_dict_from_text(text).unwrap();
        let payload = serialize_furigana_dict_to_rkyv(&dict).unwrap();
        let source_hex = {
            let digest = sha256_bytes(text.as_bytes());
            digest
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        let header = BlobHeader {
            schema_version: SCHEMA_VERSION,
            source_sha256: sha256_bytes(text.as_bytes()),
            manifest_guard: manifest_guard_from_hex_hashes(&[&source_hex]),
        };
        build_blob(&header, &payload)
    }

    #[tokio::test]
    async fn valid_blob_loads_without_touching_the_text_source() {
        // Arrange
        let provider = MockCdn::with_blob_and_text(Some(valid_blob(SAMPLE_TEXT)), SAMPLE_TEXT);

        // Act
        let loaded = load_furigana_dict_via(&provider).await.unwrap();

        // Assert: the zero-copy view answers with the same entries the
        // text source would have produced.
        let LoadedFurigana::CdnBlob(view) = &loaded else {
            panic!("expected the CDN blob fast path");
        };
        assert_eq!(view.lookup_word("指").len(), 1);
        assert!(
            !provider
                .requested_paths()
                .contains(&FURIGANA_DICT_PATH.to_string())
        );
    }

    #[tokio::test]
    async fn corrupted_header_falls_back_to_the_text_source() {
        // Arrange: valid payload, destroyed magic
        let mut blob = valid_blob(SAMPLE_TEXT);
        blob[0] = b'X';
        let provider = MockCdn::with_blob_and_text(Some(blob), SAMPLE_TEXT);

        // Act
        let loaded = load_furigana_dict_via(&provider).await.unwrap();

        // Assert
        let LoadedFurigana::SourceText(dict) = &loaded else {
            panic!("expected the text fallback");
        };
        assert_eq!(dict.lookup_word("大人").len(), 1);
    }

    #[tokio::test]
    async fn missing_blob_falls_back_to_the_text_source() {
        // Arrange
        let provider = MockCdn::with_blob_and_text(None, SAMPLE_TEXT);

        // Act
        let loaded = load_furigana_dict_via(&provider).await.unwrap();

        // Assert
        assert!(matches!(loaded, LoadedFurigana::SourceText(_)));
        assert!(
            provider
                .requested_paths()
                .contains(&FURIGANA_BLOB_PATH.to_string())
        );
    }

    #[tokio::test]
    async fn future_schema_version_falls_back_to_the_text_source() {
        // Arrange
        let mut blob = valid_blob(SAMPLE_TEXT);
        let version = SCHEMA_VERSION + 1;
        blob[4..8].copy_from_slice(&version.to_le_bytes());
        let provider = MockCdn::with_blob_and_text(Some(blob), SAMPLE_TEXT);

        // Act
        let loaded = load_furigana_dict_via(&provider).await.unwrap();

        // Assert
        assert!(matches!(loaded, LoadedFurigana::SourceText(_)));
    }
}
