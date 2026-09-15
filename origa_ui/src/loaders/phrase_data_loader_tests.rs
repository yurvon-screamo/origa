//! Unit tests for the phrase data loader core (#540-В1): chunk planning,
//! in-flight dedup across concurrent batches and retry-after-failure.
//!
//! The loader sits on process-global stores (phrase index, detail cache,
//! precompute attempted-set) — every test serializes on DATA_TEST_LOCK
//! and resets them, and the fetch assertions count only `phrases/data/`
//! requests (the best-effort precompute fetches are unstubbed misses).

use std::cell::RefCell;
use std::future::Future;
use std::sync::Mutex;

use origa::dictionary::phrase::{
    get_cached_phrase_detail, init_phrase_index, reset_phrase_data_for_test,
};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;
use ulid::Ulid;

use super::load_phrase_details_batch_via;

/// Serializes tests over the shared process-global phrase stores.
static DATA_TEST_LOCK: Mutex<()> = Mutex::new(());

const ID_A: &str = "01KPJ5S3N1DRFFD236Z4EZ03HJ"; // chunk 0
const ID_B: &str = "01KPJ5S3N1DRFFD236Z4EZ03HK"; // chunk 1

fn index_json() -> &'static str {
    r#"{"v":1,"h":"test","phrases":[
        {"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["hello"],"c":0},
        {"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["world"],"c":1}
    ]}"#
}

fn chunk_json(id: &str, text: &str) -> String {
    format!(r#"[{{"i":"{id}","x":"{text}","en":"{text}"}}]"#)
}

/// Recording mock over the CDN boundary: `failing` paths respond with a
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

    fn data_fetches(&self) -> usize {
        self.fetches
            .borrow()
            .iter()
            .filter(|path| path.starts_with("phrases/data/"))
            .count()
    }

    fn data_fetch_for(&self, chunk_file: &str) -> usize {
        self.fetches
            .borrow()
            .iter()
            .filter(|path| path.contains(chunk_file))
            .count()
    }
}

impl CdnProvider for MockCdn {
    fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
        self.fetches.borrow_mut().push(path.to_string());
        let body = self
            .bodies
            .borrow()
            .iter()
            .find(|(prefix, _)| path.starts_with(prefix.as_str()))
            .map(|(_, body)| body.clone());
        // Consume the queued failure if it targets THIS path; other
        // queued failures stay for their own request.
        let fail_now = {
            let mut failing = self.failing.borrow_mut();
            if failing.first().is_some_and(|p| p == path) {
                failing.remove(0);
                true
            } else {
                false
            }
        };
        async move {
            // A real fetch always yields at least once — this is what
            // lets the second batch reach its planning phase while the
            // first one is still in flight (the scenario the dedup
            // exists for).
            tokio::task::yield_now().await;
            match (fail_now, body) {
                (true, _) => Err(OrigaError::NetworkError {
                    url: path.to_string(),
                    reason: "stubbed failure".to_string(),
                }),
                (false, Some(body)) => Ok(body),
                (false, None) => Err(OrigaError::NetworkError {
                    url: path.to_string(),
                    reason: "not stubbed".to_string(),
                }),
            }
        }
    }

    fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
        self.fetches.borrow_mut().push(path.to_string());
        std::future::ready(Err(OrigaError::NetworkError {
            url: path.to_string(),
            reason: "not stubbed".to_string(),
        }))
    }
}

/// Fresh process-global state + the fixture index installed.
fn arrange(bodies: Vec<(String, String)>) -> (MockCdn, Vec<Ulid>) {
    reset_phrase_data_for_test();
    super::reset_phrase_precompute_chunks();
    init_phrase_index(index_json()).expect("fixture index installs");
    let provider = MockCdn::new(
        bodies
            .into_iter()
            .map(|(file, body)| (format!("phrases/data/{file}"), body))
            .collect(),
    );
    let ids = [ID_A, ID_B]
        .iter()
        .map(|id| Ulid::from_string(id).expect("valid ULID"))
        .collect();
    (provider, ids)
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global phrase stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn batch_fetches_each_missing_chunk_once_and_caches_details() {
    let _guard = DATA_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Arrange
    let (provider, ids) = arrange(vec![
        ("p0000.json".to_string(), chunk_json(ID_A, "hello text")),
        ("p0001.json".to_string(), chunk_json(ID_B, "world text")),
    ]);

    // Act
    let results = load_phrase_details_batch_via(&provider, &ids).await;

    // Assert: both ids resolved from the cache after one fetch per chunk.
    assert!(results.iter().all(|r| r.is_ok()), "all ids resolve");
    assert_eq!(provider.data_fetches(), 2, "one fetch per chunk");
    let id_a = Ulid::from_string(ID_A).unwrap();
    assert_eq!(
        get_cached_phrase_detail(&id_a).map(|d| d.text),
        Some("hello text".to_string())
    );
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global phrase stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn second_batch_does_not_refetch_cached_chunks() {
    let _guard = DATA_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Arrange
    let (provider, ids) = arrange(vec![
        ("p0000.json".to_string(), chunk_json(ID_A, "hello text")),
        ("p0001.json".to_string(), chunk_json(ID_B, "world text")),
    ]);
    load_phrase_details_batch_via(&provider, &ids)
        .await
        .unwrap_results();

    // Act
    let results = load_phrase_details_batch_via(&provider, &ids).await;

    // Assert: everything served from the cache, zero new data requests.
    assert!(results.iter().all(|r| r.is_ok()));
    assert_eq!(provider.data_fetches(), 2, "no refetch for cached chunks");
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global phrase stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn concurrent_batches_share_one_fetch_per_chunk() {
    let _guard = DATA_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Arrange: the visible-slice load and the search-driven full load
    // racing on the same chunks (two shared borrows of one provider).
    let (provider, ids) = arrange(vec![
        ("p0000.json".to_string(), chunk_json(ID_A, "hello text")),
        ("p0001.json".to_string(), chunk_json(ID_B, "world text")),
    ]);

    // Act
    let first = load_phrase_details_batch_via(&provider, &ids);
    let second = load_phrase_details_batch_via(&provider, &ids);
    let (first, second) = futures::future::join(first, second).await;

    // Assert: each chunk was fetched exactly once across both batches
    // (the in-flight dedup held) and at least one batch answered
    // everything; the loser may report Err for chunks the winner was
    // still fetching — the next batch resolves those from the cache.
    assert_eq!(
        provider.data_fetch_for("p0000.json"),
        1,
        "in-flight dedup: one fetch per chunk"
    );
    assert_eq!(provider.data_fetch_for("p0001.json"), 1);
    let all_resolved_somewhere =
        first.iter().all(|r| r.is_ok()) || second.iter().all(|r| r.is_ok());
    assert!(
        all_resolved_somewhere,
        "the batch that held the claims resolves every id"
    );

    let retry = load_phrase_details_batch_via(&provider, &ids).await;
    assert!(
        retry.iter().all(|r| r.is_ok()),
        "after the race every id resolves from the cache with no new fetches"
    );
    assert_eq!(
        provider.data_fetches(),
        2,
        "no refetch after the race settled"
    );
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global phrase stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn failed_chunk_is_retried_by_the_next_batch() {
    let _guard = DATA_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Arrange: chunk 0 fails on the first attempt, succeeds on the next.
    let (provider, ids) = arrange(vec![
        ("p0000.json".to_string(), chunk_json(ID_A, "hello text")),
        ("p0001.json".to_string(), chunk_json(ID_B, "world text")),
    ]);
    *provider.failing.borrow_mut() = vec!["phrases/data/p0000.json?v=test".to_string()];

    // Act
    let first = load_phrase_details_batch_via(&provider, &ids).await;
    let id_a = Ulid::from_string(ID_A).unwrap();
    let id_b = Ulid::from_string(ID_B).unwrap();
    assert!(first.iter().find(|r| r.is_err_at(&id_a)).is_some());

    let second = load_phrase_details_batch_via(&provider, &[id_a, id_b]).await;

    // Assert: the failure did not poison the chunk — the retry fetches
    // and resolves it.
    assert!(
        second.iter().all(|r| r.is_ok()),
        "retry must succeed after the transient failure"
    );
    assert_eq!(provider.data_fetch_for("p0000.json"), 2);
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "the std lock serializes tests over the process-global phrase stores; the awaited loads run on the test's single-threaded runtime, so no cross-task deadlock is possible"
)]
async fn unknown_id_reports_not_found_without_fetching() {
    let _guard = DATA_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Arrange: an id absent from the installed index.
    let (provider, _) = arrange(vec![]);
    let unknown = Ulid::from_string("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();

    // Act
    let results = load_phrase_details_batch_via(&provider, &[unknown]).await;

    // Assert
    assert!(matches!(
        results.first(),
        Some(Err(OrigaError::PhraseNotFound { .. }))
    ));
    assert_eq!(
        provider.data_fetches(),
        0,
        "no chunk request for an unknown id"
    );
}

/// Small test helper: panics if any result is an error.
trait UnwrapResults {
    fn unwrap_results(self);
}

impl UnwrapResults for Vec<Result<origa::dictionary::phrase::PhraseDetail, OrigaError>> {
    fn unwrap_results(self) {
        for result in self {
            result.expect("fixture batch must fully resolve");
        }
    }
}

/// Small test helper: whether this result is an error for the given id.
trait IsErrAt {
    fn is_err_at(&self, id: &Ulid) -> bool;
}

impl IsErrAt for Result<origa::dictionary::phrase::PhraseDetail, OrigaError> {
    fn is_err_at(&self, id: &Ulid) -> bool {
        match self {
            Ok(detail) => detail.id != *id,
            Err(OrigaError::PhraseNotFound { phrase_id }) => *phrase_id == *id,
            Err(_) => true,
        }
    }
}
