//! Loader tests: mock CDN, fixture index installed once per process
//! (init is idempotent), per-test chunk resets with distinct sentence ids.

use std::cell::RefCell;
use std::sync::Mutex;

use futures::future::Future;
use origa::dictionary::example::{
    chunk_id_for_sentence, reset_example_data_for_test, reset_example_index_for_test,
};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::loaders::example_loader::{
    load_example_detail_via, load_examples_via, load_word_examples_via,
};

/// Serializes tests over the process-global example stores (index OnceLock
/// + data cache): parallel tests would race resets and idempotent installs.
static EXAMPLE_TEST_LOCK: Mutex<()> = Mutex::new(());

const INDEX_JSON: &str = r#"{
    "v": 1, "h": "t", "s": 3,
    "words": {
        "本": {"refs": [[0, 0, 1], [1, 4, 6]]},
        "犬": {"refs": [[1002, -1, -1]]}
    }
}"#;

const CHUNK0_JSON: &str = r#"[
    {"i":0,"x":"本です。","en":"It's a book.","ru":"Это книга.","f":[["本","ほん"]]},
    {"i":1,"x":"これは本では?","en":"Isn't this a book?"}
]"#;

const CHUNK1_JSON: &str = r#"[
    {"i":1000,"x":"犬がいる。","en":"There's a dog."},
    {"i":1002,"x":"彼は犬を飼っている。","en":"He keeps a dog."}
]"#;

/// Recording mock over the CDN boundary. `failing` paths respond with a
/// network error exactly once, then fall back to the stubbed body.
struct MockCdn {
    bodies: RefCell<Vec<(String, String)>>,
    failing: RefCell<Vec<String>>,
    fetches: RefCell<Vec<String>>,
}

impl MockCdn {
    fn new(bodies: Vec<(String, String)>) -> Self {
        Self {
            bodies: RefCell::new(bodies),
            failing: RefCell::new(Vec::new()),
            fetches: RefCell::new(Vec::new()),
        }
    }

    fn fail_once(&self, path_prefix: &str) {
        self.failing.borrow_mut().push(path_prefix.to_string());
    }

    fn fetch_count(&self, substr: &str) -> usize {
        self.fetches
            .borrow()
            .iter()
            .filter(|p| p.contains(substr))
            .count()
    }
}

impl CdnProvider for MockCdn {
    fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
        self.fetches.borrow_mut().push(path.to_string());
        let fail_idx = self
            .failing
            .borrow()
            .iter()
            .position(|prefix| path.starts_with(prefix.as_str()));
        if let Some(idx) = fail_idx {
            self.failing.borrow_mut().remove(idx);
            return std::future::ready(Err(OrigaError::NetworkError {
                url: path.to_string(),
                reason: "injected failure".to_string(),
            }));
        }
        let hit = self
            .bodies
            .borrow()
            .iter()
            .find(|(prefix, _)| path.starts_with(prefix.as_str()))
            .map(|(_, body)| body.clone());
        std::future::ready(match hit {
            Some(body) => Ok(body),
            None => Err(OrigaError::NetworkError {
                url: path.to_string(),
                reason: "not stubbed".to_string(),
            }),
        })
    }

    fn fetch_bytes(&self, _path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
        std::future::ready(Err(OrigaError::NetworkError {
            url: _path.to_string(),
            reason: "no blobs in examples (JSON-only loader)".to_string(),
        }))
    }
}

fn mock_cdn() -> MockCdn {
    MockCdn::new(vec![
        ("examples/index.json".to_string(), INDEX_JSON.to_string()),
        ("examples/data/s0000".to_string(), CHUNK0_JSON.to_string()),
        ("examples/data/s0001".to_string(), CHUNK1_JSON.to_string()),
    ])
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global example stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn installs_index_from_cdn() {
    let _guard = EXAMPLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_example_index_for_test();
    let cdn = mock_cdn();
    load_examples_via(&cdn).await.expect("index loads");
    assert_eq!(cdn.fetch_count("examples/index.json"), 1);
    // Idempotent: second call does not refetch.
    load_examples_via(&cdn).await.expect("second call");
    assert_eq!(cdn.fetch_count("examples/index.json"), 1);
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global example stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn resolves_sentence_through_lazy_chunk() {
    let _guard = EXAMPLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_example_index_for_test();
    reset_example_data_for_test();
    // Every test installs its own index: launch order is not guaranteed.
    let cdn = mock_cdn();
    load_examples_via(&cdn).await.expect("index fixture");
    let cdn = mock_cdn();
    let detail = load_example_detail_via(&cdn, 0).await.expect("detail");
    assert_eq!(detail.text, "本です。");
    assert_eq!(detail.translation_ru.as_deref(), Some("Это книга."));
    assert_eq!(detail.ruby.len(), 1);
    assert_eq!(cdn.fetch_count("examples/data/s0000"), 1);
    // Cached: second resolution is fetch-free.
    load_example_detail_via(&cdn, 0).await.expect("cached");
    assert_eq!(cdn.fetch_count("examples/data/s0000"), 1);
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global example stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn word_examples_resolve_all_refs() {
    let _guard = EXAMPLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_example_index_for_test();
    reset_example_data_for_test();
    // Every test installs its own index: launch order is not guaranteed.
    let cdn = mock_cdn();
    load_examples_via(&cdn).await.expect("index fixture");
    let cdn = mock_cdn();
    let details = load_word_examples_via(&cdn, "本").await;
    assert_eq!(details.len(), 2);
    assert!(details.iter().all(|r| r.is_ok()));
    // Chunk ids follow sentence ids: 0 -> s0000, 1 -> s0001.
    assert_eq!(chunk_id_for_sentence(0), 0);
    assert_eq!(chunk_id_for_sentence(1), 0);
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global example stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn missing_chunk_is_an_error() {
    let _guard = EXAMPLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_example_index_for_test();
    reset_example_data_for_test();
    // Every test installs its own index: launch order is not guaranteed.
    let cdn = mock_cdn();
    load_examples_via(&cdn).await.expect("index fixture");
    let cdn = mock_cdn();
    // Sentence 5000 lives in chunk 5 — not stubbed.
    assert!(load_example_detail_via(&cdn, 5000).await.is_err());
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global example stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn unlocatable_offsets_are_preserved() {
    let _guard = EXAMPLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_example_index_for_test();
    reset_example_data_for_test();
    // Every test installs its own index: launch order is not guaranteed.
    let cdn = mock_cdn();
    load_examples_via(&cdn).await.expect("index fixture");
    let cdn = mock_cdn();
    // 犬 ref carries -1 offsets (index form != surface); sentence 1002 is in chunk 1.
    let details = load_word_examples_via(&cdn, "犬").await;
    assert_eq!(details.len(), 1);
    assert!(details[0].is_ok());
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global example stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn failed_chunk_fetch_is_retried_by_the_next_call() {
    let _guard = EXAMPLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_example_index_for_test();
    reset_example_data_for_test();
    let cdn = mock_cdn();
    cdn.fail_once("examples/data/s0000");
    // First call: transient network failure propagates...
    assert!(load_example_detail_via(&cdn, 0).await.is_err());
    // ...and the chunk is unmarked, so the next call retries the fetch.
    let detail = load_example_detail_via(&cdn, 0)
        .await
        .expect("retry succeeds");
    assert_eq!(detail.text, "本です。");
    assert_eq!(cdn.fetch_count("examples/data/s0000"), 2);
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global example stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn cached_chunk_is_not_refetched_for_other_refs() {
    let _guard = EXAMPLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset_example_index_for_test();
    reset_example_data_for_test();
    let cdn = mock_cdn();
    load_examples_via(&cdn).await.expect("index fixture");
    // 本 refs: sentences 0 and 1 — both live in chunk 0.
    let details = load_word_examples_via(&cdn, "本").await;
    assert!(details.iter().all(|r| r.is_ok()));
    assert_eq!(
        cdn.fetch_count("examples/data/s0000"),
        1,
        "one fetch per chunk"
    );
}

/// #528: the CDN audio path must match the IndexTTS batch filenames
/// byte-for-byte (`{id:06d}.opus` — see tts/gen_batch.py + tts/README.md);
/// a format drift here 404s every example clip at runtime.
#[test]
fn example_audio_path_matches_batch_pipeline_format() {
    use crate::loaders::example_loader::example_audio_path;
    assert_eq!(example_audio_path(0), "examples/audio/000000.opus");
    assert_eq!(example_audio_path(12345), "examples/audio/012345.opus");
    assert_eq!(example_audio_path(22412), "examples/audio/022412.opus");
}

/// Severity routing of chunk fetch failures pinned live (uwuwu#1454
/// review round 2, High-2 fix): online = `error!` (a Sentry event via the
/// capture bridge — the consumers swallow the `Err`, so without the log
/// the failure is fully silent), offline = `warn!`; a payload that
/// arrived but does not parse is `error!` regardless of connectivity.
/// Wasm-only: the offline branch and the capture buffer need a real
/// `navigator` and the crate's log capture.
#[cfg(all(target_arch = "wasm32", test))]
mod severity_routing_wasm {
    use super::*;
    use crate::core::log_capture;
    use crate::core::tauri_mock::OnlineOverride;
    use tracing::Level;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// Index installed from the shared fixture, then every further fetch
    /// (the chunk paths) fails with a network error.
    fn index_then_dead_cdn() -> MockCdn {
        MockCdn::new(vec![(
            "examples/index.json".to_string(),
            INDEX_JSON.to_string(),
        )])
    }

    fn reset_stores() {
        reset_example_index_for_test();
        reset_example_data_for_test();
    }

    #[wasm_bindgen_test]
    async fn chunk_fetch_failure_online_is_error() {
        let _online = OnlineOverride::install(true);
        reset_stores();
        let install = index_then_dead_cdn();
        load_examples_via(&install).await.expect("index fixture");
        install.fail_once("examples/data/");
        log_capture::install();
        log_capture::take_events();

        let result = load_example_detail_via(&install, 0).await;

        assert!(result.is_err(), "the chunk fetch must fail");
        let events = log_capture::take_events();
        assert!(
            events.iter().any(
                |(level, m)| *level == Level::ERROR && m.contains("Example chunk fetch failed")
            ),
            "online chunk failure = incident, got {events:?}"
        );
    }

    #[wasm_bindgen_test]
    async fn chunk_fetch_failure_offline_is_warn() {
        let _offline = OnlineOverride::install(false);
        reset_stores();
        let install = index_then_dead_cdn();
        load_examples_via(&install).await.expect("index fixture");
        install.fail_once("examples/data/");
        log_capture::install();
        log_capture::take_events();

        let result = load_example_detail_via(&install, 0).await;

        assert!(result.is_err(), "the chunk fetch must fail");
        let events = log_capture::take_events();
        assert!(
            events.iter().any(|(level, m)| *level == Level::WARN
                && m.contains("Example chunk fetch failed")
                && m.contains("(offline)")),
            "offline chunk failure = degradation, got {events:?}"
        );
        assert!(
            events.iter().all(|(level, _)| *level != Level::ERROR),
            "an offline failure must not emit incidents: {events:?}"
        );
    }

    #[wasm_bindgen_test]
    async fn index_fetch_failure_online_is_error() {
        // The index gates the whole examples family: an online failure (a
        // 404 / broken deploy) kills examples for the session — it must
        // report, not vanish into the caller's warn (uwuwu#1454 round 4).
        let _online = OnlineOverride::install(true);
        reset_stores();
        // No index stub: every fetch fails.
        let cdn = MockCdn::new(vec![]);
        log_capture::install();
        log_capture::take_events();

        let result = load_examples_via(&cdn).await;

        assert!(result.is_err(), "the index fetch must fail");
        let events = log_capture::take_events();
        assert!(
            events
                .iter()
                .any(|(level, m)| *level == Level::ERROR
                    && m.contains("Examples index fetch failed")),
            "online index failure = incident, got {events:?}"
        );
    }

    #[wasm_bindgen_test]
    async fn index_fetch_failure_offline_is_warn() {
        let _offline = OnlineOverride::install(false);
        reset_stores();
        let cdn = MockCdn::new(vec![]);
        log_capture::install();
        log_capture::take_events();

        let result = load_examples_via(&cdn).await;

        assert!(result.is_err(), "the index fetch must fail");
        let events = log_capture::take_events();
        assert!(
            events.iter().any(|(level, m)| *level == Level::WARN
                && m.contains("Examples index fetch failed")
                && m.contains("(offline)")),
            "offline index failure = degradation, got {events:?}"
        );
    }

    #[wasm_bindgen_test]
    async fn unparsable_payload_is_error_while_online() {
        let _online = OnlineOverride::install(true);
        reset_stores();
        let cdn = MockCdn::new(vec![
            ("examples/index.json".to_string(), INDEX_JSON.to_string()),
            (
                "examples/data/s0000".to_string(),
                String::from("not json at all"),
            ),
        ]);
        load_examples_via(&cdn).await.expect("index fixture");
        log_capture::install();
        log_capture::take_events();

        let result = load_example_detail_via(&cdn, 0).await;

        assert!(result.is_err(), "garbage payload must fail the lookup");
        let events = log_capture::take_events();
        assert!(
            events
                .iter()
                .any(|(level, m)| *level == Level::ERROR && m.contains("failed to parse/cache")),
            "an arrived-but-broken payload is a content bug: {events:?}"
        );
    }
}
