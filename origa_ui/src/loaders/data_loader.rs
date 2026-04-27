use origa::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
use origa::dictionary::kanji::{KanjiData, init_kanji, is_kanji_loaded};
use origa::dictionary::vocabulary::{VocabularyChunkData, init_vocabulary, is_vocabulary_loaded};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cdn_provider;
use crate::utils::yield_to_browser;

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

    let cdn = cdn_provider();

    let chunk_futures: Vec<_> = (1..=11)
        .map(|i| {
            let path = format!("dictionary/vocabulary/chunk_{:02}.json", i);
            async move { cdn.fetch_text(&path).await }
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

    yield_to_browser().await;
    init_vocabulary(data)?;

    tracing::info!("📖 Vocabulary loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

pub async fn load_kanji() -> Result<(), OrigaError> {
    if is_kanji_loaded() {
        tracing::debug!("📖 Kanji already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading kanji...");

    let cdn = cdn_provider();
    let json = cdn.fetch_text("dictionary/kanji.json").await?;
    let data = KanjiData { kanji_json: json };

    yield_to_browser().await;
    init_kanji(data)?;

    tracing::info!("📖 Kanji loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

pub async fn load_grammar() -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        tracing::debug!("📖 Grammar already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading grammar...");

    let cdn = cdn_provider();
    let json = cdn.fetch_text("grammar/grammar.json").await?;
    let data = GrammarData { grammar_json: json };

    yield_to_browser().await;
    init_grammar(data)?;

    tracing::info!("📖 Grammar loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}
