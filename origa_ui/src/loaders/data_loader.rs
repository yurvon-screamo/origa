use origa::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
use origa::dictionary::kanji::{KanjiData, init_kanji, is_kanji_loaded};
use origa::dictionary::radical::{RadicalData, init_radicals, is_radicals_loaded};
use origa::dictionary::vocabulary::{VocabularyChunkData, init_vocabulary, is_vocabulary_loaded};
use origa::domain::OrigaError;

use super::jlpt_content_loader::load_jlpt_content;
use crate::core::config::public_url;
use crate::repository::{
    get_cached_grammar, get_cached_kanji, get_cached_radical, get_cached_vocabulary,
    save_grammar_to_cache, save_kanji_to_cache, save_radical_to_cache, save_vocabulary_to_cache,
};
use crate::utils::fetch_text;

pub fn is_all_data_loaded() -> bool {
    is_vocabulary_loaded() && is_radicals_loaded() && is_kanji_loaded() && is_grammar_loaded()
}

pub async fn load_all_data() -> Result<(), OrigaError> {
    if is_all_data_loaded() {
        return Ok(());
    }

    let (vocab_result, radical_result, kanji_result, grammar_result, jlpt_result) = futures::join!(
        load_vocabulary(),
        load_radical(),
        load_kanji(),
        load_grammar(),
        load_jlpt_content()
    );
    vocab_result?;
    radical_result?;
    kanji_result?;
    grammar_result?;
    jlpt_result?;

    tracing::info!("All data loaded successfully");
    Ok(())
}

pub async fn load_vocabulary() -> Result<(), OrigaError> {
    use futures::future::join_all;

    if is_vocabulary_loaded() {
        return Ok(());
    }

    if let Some(cached) = get_cached_vocabulary().await? {
        init_vocabulary(cached)?;
        tracing::info!("Vocabulary loaded from cache");
        return Ok(());
    }

    let chunk_futures: Vec<_> = (1..=11)
        .map(|i| {
            fetch_text(public_url(&format!(
                "/public/dictionary/vocabulary/chunk_{:02}.json",
                i
            )))
        })
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
        chunk_11: chunks[10].clone(),
    };

    let data_for_cache = data.clone();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_vocabulary_to_cache(&data_for_cache).await {
            tracing::warn!("Failed to cache vocabulary: {:?}", e);
        }
    });

    init_vocabulary(data)?;
    tracing::info!("Vocabulary loaded from network");
    Ok(())
}

pub async fn load_radical() -> Result<(), OrigaError> {
    if is_radicals_loaded() {
        return Ok(());
    }

    if let Some(cached) = get_cached_radical().await? {
        init_radicals(cached)?;
        tracing::info!("Radicals loaded from cache");
        return Ok(());
    }

    let json = fetch_text(public_url("/public/dictionary/radicals.json")).await?;
    let data = RadicalData {
        radicals_json: json,
    };

    let data_for_cache = data.clone();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_radical_to_cache(&data_for_cache).await {
            tracing::warn!("Failed to cache radicals: {:?}", e);
        }
    });

    init_radicals(data)?;
    tracing::info!("Radicals loaded from network");
    Ok(())
}

pub async fn load_kanji() -> Result<(), OrigaError> {
    if is_kanji_loaded() {
        return Ok(());
    }

    if let Some(cached) = get_cached_kanji().await? {
        init_kanji(cached)?;
        tracing::info!("Kanji loaded from cache");
        return Ok(());
    }

    let json = fetch_text(public_url("/public/dictionary/kanji.json")).await?;
    let data = KanjiData { kanji_json: json };

    let data_for_cache = data.clone();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_kanji_to_cache(&data_for_cache).await {
            tracing::warn!("Failed to cache kanji: {:?}", e);
        }
    });

    init_kanji(data)?;
    tracing::info!("Kanji loaded from network");
    Ok(())
}

pub async fn load_grammar() -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        return Ok(());
    }

    if let Some(cached) = get_cached_grammar().await? {
        init_grammar(cached)?;
        tracing::info!("Grammar loaded from cache");
        return Ok(());
    }

    let json = fetch_text(public_url("/public/grammar/grammar.json")).await?;
    let data = GrammarData { grammar_json: json };

    let data_for_cache = data.clone();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_grammar_to_cache(&data_for_cache).await {
            tracing::warn!("Failed to cache grammar: {:?}", e);
        }
    });

    init_grammar(data)?;
    tracing::info!("Grammar loaded from network");
    Ok(())
}
