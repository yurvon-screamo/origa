use std::sync::OnceLock;

use origa::domain::JlptContent;
use origa::domain::{JapaneseLevel, OrigaError};
use serde::Deserialize;

use crate::core::config::public_url;
use crate::utils::fetch_text;

static JLPT_CONTENT: OnceLock<JlptContent> = OnceLock::new();
static DEFAULT_JLPT_CONTENT: OnceLock<JlptContent> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct KanjiDictionary {
    kanji: Vec<KanjiEntry>,
}

#[derive(Debug, Deserialize)]
struct KanjiEntry {
    kanji: String,
    jlpt: String,
}

#[derive(Debug, Deserialize)]
struct JlptWordsFile {
    #[serde(skip)]
    _level: String,
    words: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GrammarDictionary {
    grammar: Vec<GrammarEntry>,
}

#[derive(Debug, Deserialize)]
struct GrammarEntry {
    rule_id: String,
    level: String,
}

pub async fn load_jlpt_content() -> Result<(), OrigaError> {
    if JLPT_CONTENT.get().is_some() {
        tracing::debug!("📖 JLPT content already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading JLPT content...");

    let content = load_content().await?;

    let _ = JLPT_CONTENT.set(content);
    tracing::info!(
        "📖 JLPT content loaded ({:.2}s)",
        (now_ms() - start) / 1000.0
    );
    Ok(())
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

async fn load_content() -> Result<JlptContent, OrigaError> {
    let mut content = JlptContent::new();

    let start = now_ms();
    load_kanji(&mut content).await?;
    tracing::info!(
        "📖 JLPT: kanji loaded ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    let start = now_ms();
    load_words(&mut content).await?;
    tracing::info!(
        "📖 JLPT: words loaded ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    let start = now_ms();
    load_grammar(&mut content).await?;
    tracing::info!(
        "📖 JLPT: grammar loaded ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    Ok(content)
}

async fn load_kanji(content: &mut JlptContent) -> Result<(), OrigaError> {
    let start = now_ms();
    let json = fetch_text(public_url("/public/dictionary/kanji.json")).await?;
    tracing::info!(
        "📖 JLPT kanji.json fetched ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    let parse_start = now_ms();
    let data: KanjiDictionary =
        serde_json::from_str(&json).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to parse kanji.json: {}", e),
        })?;
    tracing::info!(
        "📖 JLPT kanji.json parsed ({:.2}s)",
        (now_ms() - parse_start) / 1000.0
    );

    let process_start = now_ms();
    for entry in data.kanji {
        if let Ok(level) = entry.jlpt.parse::<JapaneseLevel>() {
            content
                .kanji_by_level
                .entry(level)
                .or_default()
                .insert(entry.kanji);
        }
    }
    tracing::info!(
        "📖 JLPT kanji processed ({} entries, {:.2}s)",
        content
            .kanji_by_level
            .values()
            .map(|s| s.len())
            .sum::<usize>(),
        (now_ms() - process_start) / 1000.0
    );

    Ok(())
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
    let fetch_futures: Vec<_> = levels
        .iter()
        .map(|(_, filename)| {
            fetch_text(public_url(&format!(
                "/public/domain/well_known_set/{}",
                filename
            )))
        })
        .collect();

    let results = join_all(fetch_futures).await;
    tracing::info!(
        "📖 JLPT words files fetched ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    let process_start = now_ms();
    for (idx, (level, _)) in levels.iter().enumerate() {
        if let Ok(json) = &results[idx]
            && let Ok(data) = serde_json::from_str::<JlptWordsFile>(json)
        {
            let count = data.words.len();
            content
                .words_by_level
                .entry(*level)
                .or_default()
                .extend(data.words);
            tracing::debug!("📖 JLPT {:?} words: {}", level, count);
        }
    }
    tracing::info!(
        "📖 JLPT words processed ({} total, {:.2}s)",
        content
            .words_by_level
            .values()
            .map(|s| s.len())
            .sum::<usize>(),
        (now_ms() - process_start) / 1000.0
    );

    Ok(())
}

async fn load_grammar(content: &mut JlptContent) -> Result<(), OrigaError> {
    let start = now_ms();
    let json = fetch_text(public_url("/public/grammar/grammar.json")).await?;
    tracing::info!(
        "📖 JLPT grammar.json fetched ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    let parse_start = now_ms();
    let data: GrammarDictionary =
        serde_json::from_str(&json).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to parse grammar.json: {}", e),
        })?;
    tracing::info!(
        "📖 JLPT grammar.json parsed ({:.2}s)",
        (now_ms() - parse_start) / 1000.0
    );

    let process_start = now_ms();
    for entry in data.grammar {
        if let Ok(level) = entry.level.parse::<JapaneseLevel>() {
            content
                .grammar_by_level
                .entry(level)
                .or_default()
                .insert(entry.rule_id);
        }
    }
    tracing::info!(
        "📖 JLPT grammar processed ({} entries, {:.2}s)",
        content
            .grammar_by_level
            .values()
            .map(|s| s.len())
            .sum::<usize>(),
        (now_ms() - process_start) / 1000.0
    );

    Ok(())
}

// TODO: to domain
pub fn recalculate_user_jlpt_progress(user: &mut origa::domain::User) {
    if let Some(content) = JLPT_CONTENT.get() {
        user.recalculate_jlpt_progress(content);
    }
}

pub fn get_jlpt_content() -> &'static JlptContent {
    JLPT_CONTENT
        .get()
        .unwrap_or_else(|| DEFAULT_JLPT_CONTENT.get_or_init(JlptContent::new))
}
