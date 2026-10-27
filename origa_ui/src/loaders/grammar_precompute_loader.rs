//! Grammar precompute blob loader (#521): installs the offline-derived
//! furigana spans for every grammar markdown text node and whole string
//! so grammar pages and grammar lessons render off the tokenizer.

use origa::dictionary::cdn_blob::guard_matches;
use origa::dictionary::precompute_blob::{access_precompute_deflated, install_precompute_view};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cache_manager::guard_expectation_for;
use crate::repository::cdn_provider;

pub const GRAMMAR_PRECOMPUTE_PATH: &str = "grammar/grammar_precompute_v3.rkyv";
const GRAMMAR_SOURCES: [&str; 1] = ["grammar/grammar_v3.json"];

pub async fn load_grammar_precompute() -> Result<(), OrigaError> {
    load_grammar_precompute_via(cdn_provider()).await
}

/// Provider-parameterized core: fetch the deflated blob, verify it
/// against the manifest guard, install the zero-copy view. Any failure is
/// reported to the caller which logs and continues — grammar rendering
/// falls back to live tokenization.
pub async fn load_grammar_precompute_via<P: CdnProvider>(provider: &P) -> Result<(), OrigaError> {
    let blob = provider.fetch_bytes(GRAMMAR_PRECOMPUTE_PATH).await?;
    let (view, header) = access_precompute_deflated(&blob)?;

    let expectation = guard_expectation_for(&GRAMMAR_SOURCES);
    if !guard_matches(&header, &expectation) {
        return Err(OrigaError::GrammarParseError {
            reason: "grammar precompute blob stale relative to manifest".to_string(),
        });
    }

    install_precompute_view(view);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::future::Future;

    use super::*;
    use crate::repository::cache_manager::{CacheManifest, guard_expectation_from_manifest};
    use origa::dictionary::cdn_blob::{
        BlobHeader, GuardExpectation, SCHEMA_VERSION, build_blob, manifest_guard_from_hex_hashes,
        sha256_bytes,
    };
    use origa::dictionary::precompute_blob::{
        PrecomputeBlob, deflate_blob, installed_precompute_views_len,
        serialize_precompute_blob_to_rkyv,
    };
    use origa::domain::lookup_precomputed;
    use std::collections::{BTreeMap, HashMap};

    /// Recording mock over the external CDN boundary.
    struct MockCdn {
        blob: RefCell<Option<Vec<u8>>>,
        requested: RefCell<Vec<String>>,
    }

    impl MockCdn {
        fn with_blob(blob: Option<Vec<u8>>) -> Self {
            Self {
                blob: RefCell::new(blob),
                requested: RefCell::new(Vec::new()),
            }
        }
    }

    impl CdnProvider for MockCdn {
        fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
            self.requested.borrow_mut().push(path.to_string());
            std::future::ready(Err(OrigaError::NetworkError {
                url: path.to_string(),
                reason: "text fetch is not stubbed".to_string(),
            }))
        }

        fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
            self.requested.borrow_mut().push(path.to_string());
            let result = match (path, self.blob.borrow_mut().take()) {
                (GRAMMAR_PRECOMPUTE_PATH, Some(bytes)) => Ok(bytes),
                _ => Err(OrigaError::NetworkError {
                    url: path.to_string(),
                    reason: "not stubbed".to_string(),
                }),
            };
            std::future::ready(result)
        }
    }

    fn valid_blob(grammar_json: &str) -> Vec<u8> {
        let payload = serialize_precompute_blob_to_rkyv(&PrecomputeBlob {
            entries: BTreeMap::from([(
                "私は学生です。".to_string(),
                origa::domain::PrecomputedEntry {
                    furigana_spans: vec![origa::domain::AnnotatedSpan {
                        text: "私は学生です。".to_string(),
                        reading: Some("ワタシワガクセイデス".to_string()),
                        reading_spans: vec![],
                    }],
                    tokens: vec![],
                },
            )]),
        })
        .unwrap();
        let guard = manifest_guard_from_hex_hashes(&[&sha256_bytes(grammar_json.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()]);
        let header = BlobHeader {
            schema_version: SCHEMA_VERSION,
            source_sha256: sha256_bytes(b"sources with ingredients"),
            manifest_guard: guard,
        };
        deflate_blob(&build_blob(&header, &payload))
    }

    #[tokio::test]
    async fn valid_blob_installs_and_answers_lookups() {
        // Arrange
        let provider = MockCdn::with_blob(Some(valid_blob("{}")));

        // Act
        load_grammar_precompute_via(&provider).await.unwrap();

        // Assert: registry-relative length is deliberately not asserted —
        // sibling tests may install views concurrently; the lookup below
        // is the race-free observable outcome of THIS install.
        assert!(installed_precompute_views_len() >= 1);
        let entry = lookup_precomputed("私は学生です。").expect("installed view answers");
        assert!(
            entry.furigana_spans[0]
                .reading
                .as_deref()
                .is_some_and(|r| r.contains("ワタシ"))
        );
    }

    #[tokio::test]
    async fn corrupted_header_is_rejected() {
        // Arrange
        let mut blob = valid_blob("{}");
        blob[0] = b'X';
        let provider = MockCdn::with_blob(Some(blob));

        // Act
        let result = load_grammar_precompute_via(&provider).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn missing_blob_is_an_error_the_caller_logs() {
        // Arrange
        let provider = MockCdn::with_blob(None);

        // Act
        let result = load_grammar_precompute_via(&provider).await;

        // Assert
        assert!(result.is_err());
    }

    /// The #582 regression class: the builder derives the blob's
    /// `manifest_guard` from the manifest hashes of exactly GRAMMAR_SOURCES.
    /// A drifted source list (here or in the builder) must break this
    /// symmetry pin instead of silently rejecting every blob — the
    /// v0.7.27..0.7.37 behavior where the loader expected the retired
    /// v2/overlay pair and the precompute never installed.
    #[test]
    fn manifest_expectation_mirrors_builder_guard() {
        // Arrange: a manifest listing every loader source with the hash the
        // builder would compute for it.
        let hash = "a".repeat(64);
        let manifest = CacheManifest {
            version: 1,
            files: HashMap::from([(GRAMMAR_SOURCES[0].to_string(), hash.clone())]),
        };

        // Act: the loader-side expectation over the same constant.
        let expectation = guard_expectation_from_manifest(Some(&manifest), &GRAMMAR_SOURCES);

        // Assert: Expect(guard) equal to the builder formula.
        let GuardExpectation::Expect(guard) = expectation else {
            panic!("manifest lists every GRAMMAR_SOURCES entry — expectation must be Expect");
        };
        assert_eq!(guard, manifest_guard_from_hex_hashes(&[&hash]));

        // A foreign hash (or a source missing from the manifest) must not
        // satisfy the check.
        assert_ne!(guard, manifest_guard_from_hex_hashes(&[&"b".repeat(64)]));
        let manifest_without_source = CacheManifest {
            version: 1,
            files: HashMap::new(),
        };
        assert!(matches!(
            guard_expectation_from_manifest(Some(&manifest_without_source), &GRAMMAR_SOURCES),
            GuardExpectation::Mismatch
        ));
    }
}
