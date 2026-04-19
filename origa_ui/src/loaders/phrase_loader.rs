use origa::dictionary::phrase::{init_phrases, is_phrases_loaded};
use origa::domain::OrigaError;

use crate::core::config::public_url;
use crate::utils::{fetch_text, yield_to_browser};

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

pub async fn load_phrases() -> Result<(), OrigaError> {
    if is_phrases_loaded() {
        tracing::debug!("📖 Phrases already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading phrases...");

    let json = fetch_text(public_url("/phrase/phrase_dataset.json")).await?;

    yield_to_browser().await;
    init_phrases(&json)?;

    tracing::info!("📖 Phrases loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}
