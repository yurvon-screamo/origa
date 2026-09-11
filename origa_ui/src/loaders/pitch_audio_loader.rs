use origa::dictionary::pitch_audio::{
    get_audio_entry_count, init_pitch_audio_index, is_pitch_audio_loaded,
};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cdn_provider;
use crate::utils::{now_ms, yield_to_browser};

pub async fn load_pitch_audio() -> Result<(), OrigaError> {
    if is_pitch_audio_loaded() {
        tracing::debug!("Pitch audio already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("Loading pitch audio index...");

    let cdn = cdn_provider();
    let fetch_start = now_ms();
    let json = cdn.fetch_text("pitch/index.json").await?;
    let fetch_ms = now_ms() - fetch_start;

    yield_to_browser().await;

    let parse_start = now_ms();
    init_pitch_audio_index(&json)?;
    let parse_ms = now_ms() - parse_start;

    let total = get_audio_entry_count();

    tracing::info!(
        "Pitch audio index loaded: {} entries ({:.2}s total, fetch {:.2}s, parse {:.2}s)",
        total,
        (now_ms() - start) / 1000.0,
        fetch_ms / 1000.0,
        parse_ms / 1000.0
    );
    Ok(())
}
