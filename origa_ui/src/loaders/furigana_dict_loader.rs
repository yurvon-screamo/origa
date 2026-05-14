use crate::repository::cdn_provider;
use crate::utils::now_ms;
use origa::dictionary::furigana_dict::{init_furigana_dict, is_furigana_dict_loaded};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

const FURIGANA_DICT_PATH: &str = "dictionaries/JmdictFurigana.txt";

pub async fn load_furigana_dict() -> Result<(), OrigaError> {
    if is_furigana_dict_loaded() {
        tracing::debug!("📖 Furigana dictionary already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading furigana dictionary...");

    let cdn = cdn_provider();
    let text = cdn.fetch_text(FURIGANA_DICT_PATH).await?;

    init_furigana_dict(&text)?;

    tracing::info!(
        "📖 Furigana dictionary loaded ({:.2}s)",
        (now_ms() - start) / 1000.0
    );
    Ok(())
}
