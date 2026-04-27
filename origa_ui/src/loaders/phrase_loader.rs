use origa::dictionary::phrase::{init_phrase_index, is_phrases_loaded, iter_index_entries};
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

pub async fn load_phrases() -> Result<(), OrigaError> {
    if is_phrases_loaded() {
        tracing::debug!("Phrases already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("Loading phrases...");

    let cdn = cdn_provider();
    let json = cdn.fetch_text("phrases/phrase_index.json").await?;

    yield_to_browser().await;
    init_phrase_index(&json)?;

    let total = iter_index_entries()
        .map(|entries| entries.count())
        .unwrap_or(0);

    tracing::info!(
        "Phrases index loaded: {} phrases ({:.2}s)",
        total,
        (now_ms() - start) / 1000.0
    );
    Ok(())
}
