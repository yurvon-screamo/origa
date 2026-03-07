use std::sync::OnceLock;

use origa::domain::JlptContent;
use origa::domain::{JapaneseLevel, OrigaError};
use serde::Deserialize;
use web_sys::console;

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
    #[allow(dead_code)]
    level: String,
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
    console::info_1(&"JLPT content loaded".into());
    Ok(())
}

async fn load_content() -> Result<JlptContent, OrigaError> {
    let mut content = JlptContent::new();

    load_kanji(&mut content).await?;
    load_words(&mut content).await?;
    load_grammar(&mut content).await?;

    Ok(content)
}

async fn fetch_text(url: &str) -> Result<String, OrigaError> {
    use leptos::wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to fetch {}: {:?}", url, e),
        })?;

    let resp: web_sys::Response =
        resp_value
            .dyn_into()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to cast response for {}: {:?}", url, e),
            })?;

    if !resp.ok() {
        return Err(OrigaError::RepositoryError {
            reason: format!("Failed to fetch {}: HTTP {}", url, resp.status()),
        });
    }

    let text = JsFuture::from(resp.text().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to get text promise for {}: {:?}", url, e),
    })?)
    .await
    .map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to read text for {}: {:?}", url, e),
    })?;

    text.as_string().ok_or_else(|| OrigaError::RepositoryError {
        reason: format!("Response is not a string for {}", url),
    })
}

async fn load_kanji(content: &mut JlptContent) -> Result<(), OrigaError> {
    let json = fetch_text("/public/domain/dictionary/kanji.json").await?;
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
        let path = format!("/public/domain/well_known_set/{}", filename);
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
                console::warn_1(&format!("Warning: skipping {}: {}", path, e).into());
            }
        }
    }

    Ok(())
}

async fn load_grammar(content: &mut JlptContent) -> Result<(), OrigaError> {
    let json = fetch_text("/public/domain/grammar/grammar.json").await?;
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

pub fn recalculate_user_jlpt_progress(user: &mut origa::domain::User) {
    if let Some(content) = JLPT_CONTENT.get() {
        user.recalculate_jlpt_progress(content);
    }
}
