//! Loader tests: mock CDN, fixture index installed once per process
//! (init is idempotent), per-test chunk resets with distinct sentence ids.

use std::cell::RefCell;
use std::sync::Mutex;

use futures::future::Future;
use origa::dictionary::example::{chunk_id_for_sentence, reset_example_data_for_test};
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

/// Recording mock over the CDN boundary.
struct MockCdn {
    bodies: RefCell<Vec<(String, String)>>,
    fetches: RefCell<Vec<String>>,
}

impl MockCdn {
    fn new(bodies: Vec<(String, String)>) -> Self {
        Self {
            bodies: RefCell::new(bodies),
            fetches: RefCell::new(Vec::new()),
        }
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
    reset_example_data_for_test();
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
    reset_example_data_for_test();
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
    reset_example_data_for_test();
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
    reset_example_data_for_test();
    let cdn = mock_cdn();
    // 犬 ref carries -1 offsets (index form != surface); sentence 1002 is in chunk 1.
    let details = load_word_examples_via(&cdn, "犬").await;
    assert_eq!(details.len(), 1);
    assert!(details[0].is_ok());
}
