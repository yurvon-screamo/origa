use origa::domain::{
    GrammarData, KanjiData, OrigaError, RadicalData, VocabularyChunkData, init_grammar_rules,
    init_kanji_dictionary, init_radical_dictionary, init_vocabulary_dictionary, is_grammar_loaded,
    is_kanji_loaded, is_radical_loaded, is_vocabulary_loaded,
};

use crate::repository::load_jlpt_content;

pub fn is_all_data_loaded() -> bool {
    is_vocabulary_loaded() && is_radical_loaded() && is_kanji_loaded() && is_grammar_loaded()
}

#[cfg(target_arch = "wasm32")]
pub async fn load_all_data() -> Result<(), OrigaError> {
    if is_all_data_loaded() {
        return Ok(());
    }

    let (vocab_result, radical_result) = futures::join!(load_vocabulary(), load_radical());
    vocab_result?;
    radical_result?;

    let (kanji_result, grammar_result) = futures::join!(load_kanji(), load_grammar());
    kanji_result?;
    grammar_result?;

    load_jlpt_content().await?;

    log::info!("All data loaded successfully");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_text(url: impl Into<String>) -> Result<String, OrigaError> {
    use leptos::wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let url = format!("/public/{}", url.into());

    let window = web_sys::window().ok_or_else(|| OrigaError::TokenizerError {
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to fetch {}: {:?}", url, e),
        })?;

    let resp: web_sys::Response =
        resp_value
            .dyn_into()
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to cast response for {}: {:?}", url, e),
            })?;

    if !resp.ok() {
        return Err(OrigaError::TokenizerError {
            reason: format!("Failed to fetch {}: HTTP {}", url, resp.status()),
        });
    }

    let text = JsFuture::from(resp.text().map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to get text promise for {}: {:?}", url, e),
    })?)
    .await
    .map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read text for {}: {:?}", url, e),
    })?;

    text.as_string().ok_or_else(|| OrigaError::TokenizerError {
        reason: format!("Response is not a string for {}", url),
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn load_vocabulary() -> Result<(), OrigaError> {
    use futures::future::join_all;

    if is_vocabulary_loaded() {
        return Ok(());
    }

    let chunk_futures: Vec<_> = (1..=10)
        .map(|i| fetch_text(format!("domain/dictionary/vocabulary/chunk_{:02}.json", i)))
        .collect();

    let chunks = join_all(chunk_futures).await;
    let chunks: Vec<String> = chunks.into_iter().collect::<Result<Vec<_>, _>>()?;

    let data = VocabularyChunkData {
        chunk_01: chunks[0].clone(),
        chunk_02: chunks[1].clone(),
        chunk_03: chunks[2].clone(),
        chunk_04: chunks[3].clone(),
        chunk_05: chunks[4].clone(),
        chunk_06: chunks[5].clone(),
        chunk_07: chunks[6].clone(),
        chunk_08: chunks[7].clone(),
        chunk_09: chunks[8].clone(),
        chunk_10: chunks[9].clone(),
    };

    init_vocabulary_dictionary(data)?;
    log::info!("Vocabulary loaded");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn load_radical() -> Result<(), OrigaError> {
    if is_radical_loaded() {
        return Ok(());
    }

    let json = fetch_text("domain/dictionary/radicals.json").await?;
    init_radical_dictionary(RadicalData {
        radicals_json: json,
    })?;
    log::info!("Radicals loaded");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn load_kanji() -> Result<(), OrigaError> {
    if is_kanji_loaded() {
        return Ok(());
    }

    let json = fetch_text("domain/dictionary/kanji.json").await?;
    init_kanji_dictionary(KanjiData { kanji_json: json })?;
    log::info!("Kanji loaded");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn load_grammar() -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        return Ok(());
    }

    let json = fetch_text("domain/grammar/grammar.json").await?;
    init_grammar_rules(GrammarData { grammar_json: json })?;
    log::info!("Grammar loaded");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_all_data() -> Result<(), OrigaError> {
    if is_all_data_loaded() {
        return Ok(());
    }

    load_vocabulary()?;
    load_radical()?;
    load_kanji()?;
    load_grammar()?;
    load_jlpt_content()?;

    log::info!("All data loaded successfully");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn read_json_file(path: &str) -> Result<String, OrigaError> {
    use std::{env, fs, path::PathBuf};

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| OrigaError::TokenizerError {
        reason: "CARGO_MANIFEST_DIR not set".to_string(),
    })?;

    let full_path = PathBuf::from(manifest_dir).join("public").join(path);

    fs::read_to_string(&full_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {}", full_path.display(), e),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_vocabulary() -> Result<(), OrigaError> {
    if is_vocabulary_loaded() {
        return Ok(());
    }

    let mut chunks = Vec::with_capacity(10);
    for i in 1..=10 {
        let path = format!("domain/dictionary/vocabulary/chunk_{:02}.json", i);
        let json = read_json_file(&path)?;
        chunks.push(json);
    }

    let data = VocabularyChunkData {
        chunk_01: chunks[0].clone(),
        chunk_02: chunks[1].clone(),
        chunk_03: chunks[2].clone(),
        chunk_04: chunks[3].clone(),
        chunk_05: chunks[4].clone(),
        chunk_06: chunks[5].clone(),
        chunk_07: chunks[6].clone(),
        chunk_08: chunks[7].clone(),
        chunk_09: chunks[8].clone(),
        chunk_10: chunks[9].clone(),
    };

    init_vocabulary_dictionary(data)?;
    log::info!("Vocabulary loaded");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_radical() -> Result<(), OrigaError> {
    if is_radical_loaded() {
        return Ok(());
    }

    let json = read_json_file("domain/dictionary/radicals.json")?;
    init_radical_dictionary(RadicalData {
        radicals_json: json,
    })?;
    log::info!("Radicals loaded");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_kanji() -> Result<(), OrigaError> {
    if is_kanji_loaded() {
        return Ok(());
    }

    let json = read_json_file("domain/dictionary/kanji.json")?;
    init_kanji_dictionary(KanjiData { kanji_json: json })?;
    log::info!("Kanji loaded");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_grammar() -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        return Ok(());
    }

    let json = read_json_file("domain/grammar/grammar.json")?;
    init_grammar_rules(GrammarData { grammar_json: json })?;
    log::info!("Grammar loaded");
    Ok(())
}
