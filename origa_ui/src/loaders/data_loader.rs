use origa::dictionary::grammar::{GrammarData, init_grammar, init_grammar_from_rkyv, is_grammar_loaded};
use origa::dictionary::kanji::{KanjiData, init_kanji, init_kanji_from_rkyv, is_kanji_loaded};
use origa::dictionary::radical::{RadicalData, init_radicals, init_radicals_from_rkyv, is_radicals_loaded};
use origa::dictionary::vocabulary::{VocabularyChunkData, init_vocabulary, init_vocabulary_from_rkyv, is_vocabulary_loaded, VOCABULARY_DICTIONARY};
use origa::domain::OrigaError;

use super::jlpt_content_loader::load_jlpt_content;
use crate::core::config::public_url;
use crate::repository::{
    get_cached_grammar_rkyv, save_grammar_to_cache_rkyv,
    get_cached_kanji_rkyv, save_kanji_to_cache_rkyv,
    get_cached_radical_rkyv, save_radical_to_cache_rkyv,
    get_cached_vocabulary_rkyv, save_vocabulary_to_cache_rkyv,
};
use crate::utils::fetch_text;

pub fn is_all_data_loaded() -> bool {
    is_vocabulary_loaded() && is_radicals_loaded() && is_kanji_loaded() && is_grammar_loaded()
}

pub async fn load_all_data() -> Result<(), OrigaError> {
    if is_all_data_loaded() {
        tracing::info!("📚 All data already loaded, skipping");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📚 Starting parallel data loading...");

    let (vocab_result, radical_result, kanji_result, grammar_result, jlpt_result) = futures::join!(
        load_vocabulary(),
        load_radical(),
        load_kanji(),
        load_grammar(),
        load_jlpt_content()
    );

    let elapsed = (now_ms() - start) / 1000.0;
    tracing::info!("📚 Parallel data loading finished in {:.2}s", elapsed);

    vocab_result?;
    radical_result?;
    kanji_result?;
    grammar_result?;
    jlpt_result?;

    tracing::info!("✅ All data loaded successfully ({:.2}s total)", elapsed);
    Ok(())
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

pub async fn load_vocabulary() -> Result<(), OrigaError> {
    use futures::future::join_all;

    if is_vocabulary_loaded() {
        tracing::debug!("📖 Vocabulary already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading vocabulary...");

    // Try rkyv cache first
    if let Some(bytes) = get_cached_vocabulary_rkyv().await? {
        tracing::info!("📖 Vocabulary found in rkyv cache ({} bytes)", bytes.len());
        init_vocabulary_from_rkyv(&bytes)?;
        tracing::info!("📖 Vocabulary loaded from rkyv cache ({:.2}s)", (now_ms() - start) / 1000.0);
        return Ok(());
    }

    tracing::debug!("📖 No rkyv cache, loading from network");

    let fetch_start = now_ms();
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
    tracing::info!(
        "📖 Vocabulary chunks fetched ({:.2}s)",
        (now_ms() - fetch_start) / 1000.0
    );

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

    init_vocabulary(data)?;

    // Save to rkyv cache after initialization
    // Get the database from global and serialize it
    if let Some(db) = VOCABULARY_DICTIONARY.get() {
        let bytes = origa::dictionary::vocabulary::serialize_vocabulary_to_rkyv(db)
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to serialize vocabulary: {:?}", e),
            })?;
        
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = save_vocabulary_to_cache_rkyv(&bytes).await {
                tracing::warn!("Failed to cache vocabulary: {:?}", e);
            }
        });
    }

    tracing::info!(
        "📖 Vocabulary loaded from network ({:.2}s)",
        (now_ms() - start) / 1000.0
    );
    Ok(())
}

pub async fn load_radical() -> Result<(), OrigaError> {
    if is_radicals_loaded() {
        tracing::debug!("📖 Radicals already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading radicals...");

    // Try rkyv cache first
    if let Some(bytes) = get_cached_radical_rkyv().await? {
        tracing::info!("📖 Radicals found in rkyv cache ({} bytes)", bytes.len());
        init_radicals_from_rkyv(&bytes)?;
        tracing::info!("📖 Radicals loaded from rkyv cache ({:.2}s)", (now_ms() - start) / 1000.0);
        return Ok(());
    }

    tracing::debug!("📖 No rkyv cache, loading from network");

    let json = fetch_text(public_url("/public/dictionary/radicals.json")).await?;
    let data = RadicalData { radicals_json: json };

    // Serialize before init (takes reference, doesn't move data)
    let bytes = origa::dictionary::radical::serialize_radicals_to_rkyv(&data)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to serialize radicals: {:?}", e),
        })?;

    // Now init takes ownership
    init_radicals(data)?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_radical_to_cache_rkyv(&bytes).await {
            tracing::warn!("Failed to cache radicals: {:?}", e);
        }
    });

    tracing::info!(
        "📖 Radicals loaded from network ({:.2}s)",
        (now_ms() - start) / 1000.0
    );
    Ok(())
}

pub async fn load_kanji() -> Result<(), OrigaError> {
    if is_kanji_loaded() {
        tracing::debug!("📖 Kanji already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading kanji...");

    // Try rkyv cache first
    if let Some(bytes) = get_cached_kanji_rkyv().await? {
        tracing::info!("📖 Kanji found in rkyv cache ({} bytes)", bytes.len());
        init_kanji_from_rkyv(&bytes)?;
        tracing::info!("📖 Kanji loaded from rkyv cache ({:.2}s)", (now_ms() - start) / 1000.0);
        return Ok(());
    }

    tracing::debug!("📖 No rkyv cache, loading from network");

    let json = fetch_text(public_url("/public/dictionary/kanji.json")).await?;
    let data = KanjiData { kanji_json: json };

    // Serialize before init (takes reference, doesn't move data)
    let bytes = origa::dictionary::kanji::serialize_kanji_to_rkyv(&data)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to serialize kanji: {:?}", e),
        })?;

    // Now init takes ownership
    init_kanji(data)?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_kanji_to_cache_rkyv(&bytes).await {
            tracing::warn!("Failed to cache kanji: {:?}", e);
        }
    });

    tracing::info!(
        "📖 Kanji loaded from network ({:.2}s)",
        (now_ms() - start) / 1000.0
    );
    Ok(())
}

pub async fn load_grammar() -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        tracing::debug!("📖 Grammar already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading grammar...");

    // Try rkyv cache first
    if let Some(bytes) = get_cached_grammar_rkyv().await? {
        tracing::info!("📖 Grammar found in rkyv cache ({} bytes)", bytes.len());
        init_grammar_from_rkyv(&bytes)?;
        tracing::info!("📖 Grammar loaded from rkyv cache ({:.2}s)", (now_ms() - start) / 1000.0);
        return Ok(());
    }

    tracing::debug!("📖 No rkyv cache, loading from network");

    let json = fetch_text(public_url("/public/grammar/grammar.json")).await?;
    let data = GrammarData { grammar_json: json };

    // Serialize before init (takes reference, doesn't move data)
    let bytes = origa::dictionary::grammar::serialize_grammar_to_rkyv(&data)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to serialize grammar: {:?}", e),
        })?;

    // Now init takes ownership
    init_grammar(data)?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = save_grammar_to_cache_rkyv(&bytes).await {
            tracing::warn!("Failed to cache grammar: {:?}", e);
        }
    });

    tracing::info!(
        "📖 Grammar loaded from network ({:.2}s)",
        (now_ms() - start) / 1000.0
    );
    Ok(())
}
