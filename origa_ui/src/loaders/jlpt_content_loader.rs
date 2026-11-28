use std::sync::OnceLock;

use origa::dictionary::grammar::GRAMMAR_RULES;
use origa::dictionary::kanji::KANJI_DICTIONARY;
use origa::domain::JlptContent;
use origa::domain::{JapaneseLevel, OrigaError};
use origa::traits::CdnProvider;
use serde::Deserialize;

use crate::repository::cdn_provider;
use crate::utils::now_ms;

static JLPT_CONTENT: OnceLock<JlptContent> = OnceLock::new();
static DEFAULT_JLPT_CONTENT: OnceLock<JlptContent> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct JlptWordsFile {
    #[serde(skip)]
    _level: String,
    words: Vec<String>,
}

/// One aggregated severity-routed report for the level files
/// (uwuwu#1454): five files fetch in parallel, a dead CDN must not emit
/// five events for one root cause. Testable seam for the wasm suite.
fn report_jlpt_fetch_failures(failures: &[String], total: usize) {
    if failures.is_empty() {
        return;
    }
    crate::tracing_console::report_network_failure(&format!(
        "JLPT words fetches failed ({}/{}): {}",
        failures.len(),
        total,
        failures.join("; ")
    ));
}

pub async fn load_jlpt_content() -> Result<(), OrigaError> {
    if JLPT_CONTENT.get().is_some() {
        tracing::debug!("JLPT content already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("Loading JLPT content...");

    let content = load_content().await?;

    let _ = JLPT_CONTENT.set(content);
    tracing::info!("JLPT content loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

async fn load_content() -> Result<JlptContent, OrigaError> {
    let mut content = JlptContent::new();

    let start = now_ms();
    build_kanji_index(&mut content);
    tracing::info!(
        "JLPT: kanji index built from domain ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    let start = now_ms();
    build_grammar_index(&mut content);
    tracing::info!(
        "JLPT: grammar index built from domain ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    let start = now_ms();
    load_words(&mut content).await?;
    tracing::info!("JLPT: words loaded ({:.2}s)", (now_ms() - start) / 1000.0);

    let start = now_ms();
    build_counters_index(&mut content);
    tracing::info!(
        "JLPT: counters indexed ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    Ok(content)
}

fn build_kanji_index(content: &mut JlptContent) {
    let Some(db) = KANJI_DICTIONARY.get() else {
        tracing::warn!("JLPT: kanji dictionary not loaded, skipping kanji index");
        return;
    };

    for level in JapaneseLevel::ALL {
        let kanji_list = db.get_kanji_list(&level);
        let set: std::collections::HashSet<String> =
            kanji_list.iter().map(|k| k.kanji().to_string()).collect();
        if !set.is_empty() {
            content.kanji_by_level.insert(level, set);
        }
    }

    tracing::info!(
        "JLPT kanji indexed ({} entries)",
        content
            .kanji_by_level
            .values()
            .map(|s| s.len())
            .sum::<usize>()
    );
}

fn build_grammar_index(content: &mut JlptContent) {
    let Some(rules) = GRAMMAR_RULES.get() else {
        tracing::warn!("JLPT: grammar rules not loaded, skipping grammar index");
        return;
    };

    for rule in rules.iter() {
        content
            .grammar_by_level
            .entry(*rule.level())
            .or_default()
            .insert(rule.rule_id().to_string());
    }

    tracing::info!(
        "JLPT grammar indexed ({} entries)",
        content
            .grammar_by_level
            .values()
            .map(|s| s.len())
            .sum::<usize>()
    );
}

fn build_counters_index(content: &mut JlptContent) {
    let counters = origa::dictionary::counters::COUNTERS.get();
    match counters {
        Some(entries) if !entries.is_empty() => {
            for entry in entries {
                content
                    .counters_by_level
                    .entry(entry.level())
                    .or_default()
                    .insert(entry.suffix().to_string());
            }
            tracing::info!(
                "JLPT counters indexed ({} entries)",
                content
                    .counters_by_level
                    .values()
                    .map(|s| s.len())
                    .sum::<usize>()
            );
        },
        _ => tracing::warn!("JLPT: counters registry not loaded, skipping counter index"),
    }
}

async fn load_words(content: &mut JlptContent) -> Result<(), OrigaError> {
    use futures::future::join_all;

    let levels = [
        (JapaneseLevel::N5, "jlpt_n5.json"),
        (JapaneseLevel::N4, "jlpt_n4.json"),
        (JapaneseLevel::N3, "jlpt_n3.json"),
        (JapaneseLevel::N2, "jlpt_n2.json"),
        (JapaneseLevel::N1, "jlpt_n1.json"),
    ];

    let start = now_ms();
    let cdn = cdn_provider();
    let fetch_futures: Vec<_> = levels
        .iter()
        .map(|(_, filename)| {
            let path = format!("well_known_set/{}", filename);
            async move { cdn.fetch_text(&path).await }
        })
        .collect();

    let results = join_all(fetch_futures).await;
    tracing::info!(
        "JLPT words files fetched ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    let process_start = now_ms();
    // Fetch failures aggregate into ONE severity-routed report: five level
    // files fetch in parallel, a dead CDN would otherwise emit five events
    // for one root cause (uwuwu#1454 review).
    let mut fetch_failures: Vec<String> = Vec::new();
    for (idx, (level, filename)) in levels.iter().enumerate() {
        match &results[idx] {
            Ok(json) => match serde_json::from_str::<JlptWordsFile>(json) {
                Ok(data) => {
                    let count = data.words.len();
                    content
                        .words_by_level
                        .entry(*level)
                        .or_default()
                        .extend(data.words);
                    tracing::info!("JLPT {:?} words loaded: {}", level, count);
                },
                Err(e) => {
                    tracing::error!(
                        "JLPT {:?} words parse error for {}: {:?}",
                        level,
                        filename,
                        e
                    );
                },
            },
            Err(e) => {
                // Collected for the single aggregated report below; parse
                // failures above stay per-level `error!` — a fetched
                // payload that does not parse is a content bug regardless
                // of connectivity.
                fetch_failures.push(format!("{level:?} {}: {e:?}", filename));
            },
        }
    }
    report_jlpt_fetch_failures(&fetch_failures, levels.len());
    tracing::info!(
        "JLPT words processed ({} total, {:.2}s)",
        content
            .words_by_level
            .values()
            .map(|s| s.len())
            .sum::<usize>(),
        (now_ms() - process_start) / 1000.0
    );

    let total_words: usize = content.words_by_level.values().map(|s| s.len()).sum();
    if total_words == 0 {
        return Err(OrigaError::RepositoryError {
            reason: "No JLPT words loaded from CDN — all fetches failed".to_string(),
        });
    }

    Ok(())
}

pub fn recalculate_user_jlpt_progress(user: &mut origa::domain::User) {
    if let Some(content) = JLPT_CONTENT.get() {
        let n5_words = content.total_words(origa::domain::JapaneseLevel::N5);
        let n4_words = content.total_words(origa::domain::JapaneseLevel::N4);
        let known_count = user
            .knowledge_set()
            .study_cards()
            .values()
            .filter(|sc| sc.memory().is_known_card())
            .count();
        tracing::info!(
            "Recalculating JLPT progress: N5 words={}, N4 words={}, known cards={}",
            n5_words,
            n4_words,
            known_count,
        );
        user.recalculate_jlpt_progress(content);
    } else {
        tracing::warn!("JLPT content not loaded, skipping progress recalculation");
    }
}

pub fn get_jlpt_content() -> &'static JlptContent {
    JLPT_CONTENT
        .get()
        .unwrap_or_else(|| DEFAULT_JLPT_CONTENT.get_or_init(JlptContent::new))
}

/// The aggregated report pinned live (uwuwu#1454 round 4): five level
/// files, one dead CDN — one severity-routed event, not five.
#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use crate::core::log_capture;
    use crate::core::tauri_mock::OnlineOverride;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn failures(n: usize) -> Vec<String> {
        (0..n)
            .map(|i| format!("N{}/jlpt_n{}.json: dead", 5 - i, 5 - i))
            .collect()
    }

    #[wasm_bindgen_test]
    fn fetch_failures_online_are_one_error_event() {
        let _online = OnlineOverride::install(true);
        log_capture::install();
        log_capture::take_events();
        report_jlpt_fetch_failures(&failures(5), 5);
        let events = log_capture::take_events();
        assert_eq!(events.len(), 1, "one aggregated event, got {events:?}");
        assert_eq!(events[0].0, tracing::Level::ERROR);
        let m = &events[0].1;
        assert!(m.contains("JLPT words fetches failed (5/5)"), "{m}");
        assert!(m.contains("jlpt_n5.json"), "carries the file names: {m}");
    }

    #[wasm_bindgen_test]
    fn fetch_failures_offline_are_one_warn_event() {
        let _offline = OnlineOverride::install(false);
        log_capture::install();
        log_capture::take_events();
        report_jlpt_fetch_failures(&failures(3), 5);
        let events = log_capture::take_events();
        assert_eq!(events.len(), 1, "got {events:?}");
        assert_eq!(events[0].0, tracing::Level::WARN);
        assert!(events[0].1.contains("(offline)"), "{}", events[0].1);
    }

    #[wasm_bindgen_test]
    fn no_failures_emit_nothing() {
        let _online = OnlineOverride::install(true);
        log_capture::install();
        log_capture::take_events();
        report_jlpt_fetch_failures(&[], 5);
        assert!(log_capture::take_events().is_empty());
    }
}
