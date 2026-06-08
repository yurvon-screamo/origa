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
                tracing::error!(
                    "JLPT {:?} words fetch failed for {}: {:?}",
                    level,
                    filename,
                    e
                );
            },
        }
    }
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
