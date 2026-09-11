use origa::dictionary::phrase::{init_phrase_index, is_phrases_loaded, iter_index_entries};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cdn_provider;
use crate::utils::{now_ms, yield_to_browser};

pub async fn load_phrases() -> Result<(), OrigaError> {
    if is_phrases_loaded() {
        tracing::debug!("Phrases already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("Loading phrases...");

    let cdn = cdn_provider();
    let fetch_start = now_ms();
    let json = cdn.fetch_text("phrases/phrase_index.json").await?;
    let fetch_ms = now_ms() - fetch_start;

    yield_to_browser().await;

    let parse_start = now_ms();
    init_phrase_index(&json)?;
    let parse_ms = now_ms() - parse_start;

    let total = iter_index_entries()
        .map(|entries| entries.count())
        .unwrap_or(0);

    tracing::info!(
        "Phrases index loaded: {} phrases ({:.2}s total, fetch {:.2}s, parse {:.2}s)",
        total,
        (now_ms() - start) / 1000.0,
        fetch_ms / 1000.0,
        parse_ms / 1000.0
    );
    Ok(())
}
