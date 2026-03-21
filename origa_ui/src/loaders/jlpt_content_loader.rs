use std::sync::OnceLock;

use origa::domain::JlptContent;
use origa::domain::{JapaneseLevel, OrigaError};
use serde::Deserialize;

use crate::core::config::public_url;
use crate::utils::fetch_text;

static JLPT_CONTENT: OnceLock<JlptContent> = OnceLock::new();

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
        return Ok(());
    }

    let content = load_content().await?;

    let _ = JLPT_CONTENT.set(content);
    tracing::info!("JLPT content loaded");
    Ok(())
}

async fn load_content() -> Result<JlptContent, OrigaError> {
    let mut content = JlptContent::new();

    load_kanji(&mut content).await?;
    load_words(&mut content).await?;
    load_grammar(&mut content).await?;

    Ok(content)
}

async fn load_kanji(content: &mut JlptContent) -> Result<(), OrigaError> {
    let json = fetch_text(public_url("/public/dictionary/kanji.json")).await?;
    let data: KanjiDictionary =
        serde_json::from_str(&json).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to parse kanji.json: {}", e),
        })?;

    for entry in data.kanji {
        if let Ok(level) = entry.jlpt.parse::<JapaneseLevel>() {
            content
                .kanji_by_level
                .entry(level)
                .or_default()
                .insert(entry.kanji);
        }
    }

    Ok(())
}

async fn load_words(content: &mut JlptContent) -> Result<(), OrigaError> {
    let levels = [
        (JapaneseLevel::N5, "jltp_n5.json"),
        (JapaneseLevel::N4, "jltp_n4.json"),
        (JapaneseLevel::N3, "jltp_n3.json"),
        (JapaneseLevel::N2, "jltp_n2.json"),
        (JapaneseLevel::N1, "jltp_n1.json"),
    ];

    for (level, filename) in levels {
        let path = public_url(&format!("/public/domain/well_known_set/{}", filename));
        match fetch_text(&path).await {
            Ok(json) => {
                if let Ok(data) = serde_json::from_str::<JlptWordsFile>(&json) {
                    content
                        .words_by_level
                        .entry(level)
                        .or_default()
                        .extend(data.words);
                }
            }
            Err(e) => {
                tracing::warn!("Warning: skipping {}: {}", path, e);
            }
        }
    }

    Ok(())
}

async fn load_grammar(content: &mut JlptContent) -> Result<(), OrigaError> {
    let json = fetch_text(public_url("/public/grammar/grammar.json")).await?;
    let data: GrammarDictionary =
        serde_json::from_str(&json).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to parse grammar.json: {}", e),
        })?;

    for entry in data.grammar {
        if let Ok(level) = entry.level.parse::<JapaneseLevel>() {
            content
                .grammar_by_level
                .entry(level)
                .or_default()
                .insert(entry.rule_id);
        }
    }

    Ok(())
}

// TODO: to domain
pub fn recalculate_user_jlpt_progress(user: &mut origa::domain::User) {
    if let Some(content) = JLPT_CONTENT.get() {
        user.recalculate_jlpt_progress(content);
    }
}
