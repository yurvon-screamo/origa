use origa::dictionary::pitch_audio::{
    get_audio_entry_count, init_pitch_audio_index, is_pitch_audio_loaded,
};
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

pub async fn load_pitch_audio() -> Result<(), OrigaError> {
    if is_pitch_audio_loaded() {
        tracing::debug!("Pitch audio already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("Loading pitch audio index...");

    let cdn = cdn_provider();
    let json = cdn.fetch_text("pitch/index.json").await?;

    yield_to_browser().await;
    init_pitch_audio_index(&json)?;

    let total = get_audio_entry_count();

    tracing::info!(
        "Pitch audio index loaded: {} entries ({:.2}s)",
        total,
        (now_ms() - start) / 1000.0
    );
    Ok(())
}
