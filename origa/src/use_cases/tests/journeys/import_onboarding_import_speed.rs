//! Import speed journeys: the "very long import" bug (owner scenario, 184
//! sets / ~9k words). The CPU path is already linear (see
//! `import_onboarding_n2_scale`); these tests pin the remaining contract:
//! CDN fetches run concurrently, wall time stays inside the measured budget,
//! errors surface in input order, and progress events cover every phase.
//!
//! Budget provenance (10.11 Design measurement, native debug): full owner
//! scenario ≈ 0.95 s → 4 s ceiling leaves CI-runner headroom without
//! passing a sequential fetch regression (5 s+ under the same latency mock).

use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError, User};
use crate::traits::{CdnProvider, UserRepository};
use crate::use_cases::tests::fixtures::{
    InMemoryUserRepository, MockCdnProvider, get_cdn_dir, init_real_dictionaries,
};
use crate::use_cases::{ImportOnboardingSetsUseCase, ImportPhase, ImportProgressEvent};

/// CDN mock that delays every fetch (simulated network round trip) and
/// tracks how many requests are in flight at once — the observable form of
/// "fetches run concurrently".
struct LatencyCdn {
    texts: HashMap<String, String>,
    delay_ms: u64,
    /// Extra per-path delay (ms) — makes completion order diverge from input
    /// order deterministically in the error-ordering test.
    extra_delay_ms: HashMap<String, u64>,
    in_flight: Arc<AtomicUsize>,
    max_in_flight: Arc<AtomicUsize>,
}

impl LatencyCdn {
    fn with_corpus(ids: &[&str], delay_ms: u64) -> Self {
        let mut texts = HashMap::new();
        for id in ids {
            let body =
                std::fs::read_to_string(get_cdn_dir().join(crate::domain::resolve_set_path(id)))
                    .unwrap_or_else(|e| {
                        panic!(
                            "{} must exist in cdn/: {e}",
                            crate::domain::resolve_set_path(id)
                        )
                    });
            texts.insert(crate::domain::resolve_set_path(id), body);
        }
        Self {
            texts,
            delay_ms,
            extra_delay_ms: HashMap::new(),
            in_flight: Arc::new(AtomicUsize::new(0)),
            max_in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn with_extra_delay(mut self, path_part: &str, extra_ms: u64) -> Self {
        self.extra_delay_ms.insert(path_part.to_string(), extra_ms);
        self
    }
}

impl CdnProvider for LatencyCdn {
    fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
        let body = self.texts.get(path).cloned();
        let mut delay = self.delay_ms;
        for (part, extra) in &self.extra_delay_ms {
            if path.contains(part) {
                delay += extra;
            }
        }
        let delay = Duration::from_millis(delay);
        let in_flight = self.in_flight.clone();
        let max_in_flight = self.max_in_flight.clone();
        async move {
            let current = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            max_in_flight.fetch_max(current, Ordering::SeqCst);
            tokio::time::sleep(delay).await;
            in_flight.fetch_sub(1, Ordering::SeqCst);
            body.ok_or_else(|| OrigaError::NetworkError {
                url: path.to_string(),
                reason: "Not found in mock".to_string(),
            })
        }
    }

    fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
        std::future::ready(Err(OrigaError::NetworkError {
            url: path.to_string(),
            reason: "bytes not used by set import".to_string(),
        }))
    }
}

/// The owner's 09.11 selection: DuolingoEn N3 (92) + Irodori N5 (18) +
/// JLPT n5/n4/n3 (3) + Migii N5 (20) + Minna N5 (25) + Minna N4 (25) +
/// first Migii N4 (1) = 184 sets, ~9.2k words (screen said 8663 — minor
/// metadata drift is acceptable for a speed contract).
fn owner_set_ids() -> Vec<String> {
    let meta_path = get_cdn_dir().join("well_known_set/well_known_sets_meta.json");
    let meta: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(&meta_path).unwrap()).unwrap();
    let pick = |pred: &dyn Fn(&serde_json::Value) -> bool| -> Vec<String> {
        let mut ids: Vec<String> = meta
            .iter()
            .filter(|m| pred(m))
            .map(|m| m["id"].as_str().unwrap().to_string())
            .collect();
        ids.sort();
        ids
    };
    let mut ids = Vec::new();
    ids.extend(pick(&|m| {
        m["set_type"] == "DuolingoEn" && m["level"] == "N3"
    }));
    ids.extend(pick(&|m| m["set_type"] == "Irodori" && m["level"] == "N5"));
    ids.extend(
        ["jlpt_n5", "jlpt_n4", "jlpt_n3"]
            .iter()
            .map(|s| s.to_string()),
    );
    ids.extend(pick(&|m| m["set_type"] == "Migii" && m["level"] == "N5"));
    ids.extend(pick(&|m| {
        m["set_type"] == "MinnaNoNihongo" && m["level"] == "N5"
    }));
    ids.extend(pick(&|m| {
        m["set_type"] == "MinnaNoNihongo" && m["level"] == "N4"
    }));
    let extra = pick(&|m| m["set_type"] == "Migii" && m["level"] == "N4");
    ids.push(extra[0].clone());
    assert_eq!(
        ids.len(),
        184,
        "the scenario mirrors the owner's 184-set selection"
    );
    ids
}

fn corpus_cdn(ids: &[String]) -> MockCdnProvider {
    let mut cdn = MockCdnProvider::new();
    for id in ids {
        let rel = crate::domain::resolve_set_path(id);
        let body = std::fs::read_to_string(get_cdn_dir().join(&rel))
            .unwrap_or_else(|e| panic!("{rel} must exist in cdn/: {e}"));
        cdn = cdn.with_text(&rel, &body);
    }
    cdn
}

fn fresh_repo() -> InMemoryUserRepository {
    InMemoryUserRepository::with_user(User::new(
        "import-speed@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ))
}

/// Simulated network round trips must overlap: a sequential fetch loop
/// never has two requests in flight (the "very long import" bug).
#[tokio::test]
async fn parallel_fetch_runs_set_requests_concurrently() {
    init_real_dictionaries();
    // Arrange — 16 sets at 50 ms round trip each: sequential = 800 ms of
    // pure waiting, bounded-parallel = a couple of batch rounds.
    let ids: Vec<String> = [
        "jlpt_n5",
        "jlpt_n4",
        "jlpt_n3",
        "jlpt_n2",
        "jlpt_n1",
        "irodori_nyuumon_01",
        "irodori_nyuumon_02",
        "irodori_nyuumon_03",
        "irodori_nyuumon_04",
        "irodori_nyuumon_05",
        "minna_n5_1",
        "minna_n5_2",
        "migii_n5_1",
        "migii_n5_2",
        "migii_n5_3",
        "migii_n5_4",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let cdn = LatencyCdn::with_corpus(&ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(), 50);
    let repo = fresh_repo();
    let use_case = ImportOnboardingSetsUseCase::new(&repo, &cdn);

    // Act
    let result = use_case
        .execute(
            repo.get_current_user().await.unwrap().unwrap(),
            ids,
            JapaneseLevel::N5,
        )
        .await;

    // Assert
    let result = result.expect("import with a complete mock corpus must succeed");
    assert!(result.created_vocabulary > 0, "sets must produce cards");
    assert!(
        cdn.max_in_flight.load(Ordering::SeqCst) >= 2,
        "set fetches must overlap — max in flight was {}",
        cdn.max_in_flight.load(Ordering::SeqCst)
    );
}

/// The owner scenario under a 25 ms simulated round trip must finish within
/// the measured budget. Sequential fetching alone costs 184 × 25 ms ≈ 4.6 s
/// — over budget even before any CPU work.
///
/// Best-of-3 wall time, same rationale as `fastest_import` in
/// `import_onboarding_n2_scale`: the minimum is robust against asymmetric
/// CPU contention from parallel tests in the shared test pool.
#[tokio::test]
async fn owner_scale_import_with_network_latency_stays_within_budget() {
    init_real_dictionaries();
    // Arrange
    let ids = owner_set_ids();
    let cdn = LatencyCdn::with_corpus(&ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(), 25);

    // Act — best-of-3, a fresh user per run (the import saves into the
    // repository; re-importing into the previous user would dedup to zero)
    let mut best = Duration::MAX;
    let mut created = 0;
    for _ in 0..3 {
        let repo = fresh_repo();
        let use_case = ImportOnboardingSetsUseCase::new(&repo, &cdn);
        let user = repo.get_current_user().await.unwrap().unwrap();
        let started = Instant::now();
        let result = use_case.execute(user, ids.clone(), JapaneseLevel::N3).await;
        let elapsed = started.elapsed();
        let result = result.expect("import with a complete mock corpus must succeed");
        created = result.created_vocabulary + result.created_kanji;
        best = best.min(elapsed);
    }

    // Assert
    assert!(created > 5000, "owner scale must create the full card set");
    assert!(
        best < Duration::from_secs(4),
        "import with 25 ms round trips must stay within the 4 s budget, took {best:?}"
    );
}

/// Zero-latency absolute ceiling for the CPU path — pins the debug-profile
/// cost of the full owner scenario so any regression shows up as a budget
/// breach, not as a slow app on a real device. Best-of-3 for the same
/// contention-robustness reason as the latency budget above.
#[tokio::test]
async fn owner_scale_import_cpu_stays_within_budget() {
    init_real_dictionaries();
    // Arrange
    let ids = owner_set_ids();
    let cdn = corpus_cdn(&ids);

    // Act — best-of-3, a fresh user per run (the import saves into the
    // repository; re-importing into the previous user would dedup to zero)
    let mut best = Duration::MAX;
    let mut created = 0;
    for _ in 0..3 {
        let repo = fresh_repo();
        let use_case = ImportOnboardingSetsUseCase::new(&repo, &cdn);
        let user = repo.get_current_user().await.unwrap().unwrap();
        let started = Instant::now();
        let result = use_case.execute(user, ids.clone(), JapaneseLevel::N3).await;
        let elapsed = started.elapsed();
        let result = result.expect("import with a complete mock corpus must succeed");
        created = result.created_vocabulary + result.created_kanji;
        best = best.min(elapsed);
    }

    // Assert
    assert!(created > 5000);
    assert!(
        best < Duration::from_secs(4),
        "debug-profile CPU import of the owner scenario must stay within 4 s, took {best:?}"
    );
}

/// Parallel fetching must not change observable outcomes: with two failing
/// sets completing in the opposite order of the input, the reported error is
/// still the input-order-first one (identical to the sequential behavior).
#[tokio::test]
async fn import_reports_the_input_order_first_set_error() {
    init_real_dictionaries();
    // Arrange — good sets fetch instantly, the late-order failure
    // (irodori_missing_late) resolves before the early-order one
    // (irodori_missing_early): completion order differs from input order.
    let ids = [
        "jlpt_n5".to_string(),
        "irodori_missing_early".to_string(),
        "irodori_n5_1".to_string(),
        "irodori_missing_late".to_string(),
    ];
    let cdn = LatencyCdn::with_corpus(&["jlpt_n5", "irodori_nyuumon_01"], 5)
        // The late-order failure resolves first; the early-order one hangs
        // one round longer — completion order differs from input order.
        .with_extra_delay("irodori_missing_early", 80);
    let repo = fresh_repo();
    let use_case = ImportOnboardingSetsUseCase::new(&repo, &cdn);

    // Act
    let error = use_case
        .execute(
            repo.get_current_user().await.unwrap().unwrap(),
            ids.to_vec(),
            JapaneseLevel::N5,
        )
        .await
        .expect_err("import with missing set bodies must fail");

    // Assert
    let rendered = error.to_string();
    assert!(
        rendered.contains("irodori_missing_early"),
        "the input-order-first failing set must be reported, got: {rendered}"
    );
}

/// Progress events cover every phase in order, count monotonically inside a
/// phase, and end with the save phase.
#[tokio::test]
async fn progress_events_cover_every_phase_in_order() {
    init_real_dictionaries();
    // Arrange
    let ids: Vec<String> = ["jlpt_n5", "jlpt_n4"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let cdn = corpus_cdn(&ids);
    let repo = fresh_repo();
    let use_case = ImportOnboardingSetsUseCase::new(&repo, &cdn);
    let events: Arc<Mutex<Vec<ImportProgressEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();

    // Act
    use_case
        .execute_with_progress(
            repo.get_current_user().await.unwrap().unwrap(),
            ids,
            JapaneseLevel::N4,
            |event: ImportProgressEvent| {
                let sink = sink.clone();
                async move {
                    sink.lock().unwrap().push(event);
                }
            },
        )
        .await
        .expect("import with a complete mock corpus must succeed");

    // Assert
    let events = events.lock().unwrap().clone();
    assert!(
        !events.is_empty(),
        "progress must emit at least one event per phase"
    );

    let phase_order: Vec<ImportPhase> = {
        let mut seen: Vec<ImportPhase> = Vec::new();
        for event in &events {
            if seen.last() != Some(&event.phase) {
                seen.push(event.phase);
            }
        }
        seen
    };
    assert_eq!(
        phase_order,
        vec![
            ImportPhase::FetchSets,
            ImportPhase::Vocabulary,
            ImportPhase::Grammar,
            ImportPhase::Kanji,
            ImportPhase::Saving,
        ],
        "phases must appear in pipeline order without regressions"
    );

    let fetch_events: Vec<&ImportProgressEvent> = events
        .iter()
        .filter(|e| e.phase == ImportPhase::FetchSets)
        .collect();
    assert_eq!(
        fetch_events.last().map(|e| e.total),
        Some(2),
        "fetch totals track the selected set count"
    );
    let mut previous_done = 0;
    for event in &fetch_events {
        assert!(
            event.done > previous_done,
            "fetch progress must move forward"
        );
        previous_done = event.done;
    }

    let saving = events.last().unwrap();
    assert_eq!(
        saving.phase,
        ImportPhase::Saving,
        "the save phase closes the import"
    );
}
