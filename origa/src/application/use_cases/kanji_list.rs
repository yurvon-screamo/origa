use tracing::{debug, info};

use crate::domain::{JapaneseLevel, KanjiInfo, OrigaError, get_kanji_list};

pub struct KanjiListUseCase;

impl Default for KanjiListUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl KanjiListUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, level: &JapaneseLevel) -> Result<Vec<KanjiInfo>, OrigaError> {
        debug!(level = ?level, "Getting kanji list");

        let result: Vec<KanjiInfo> = get_kanji_list(level)
            .into_iter()
            .map(|x| (*x).clone())
            .collect();

        info!(count = result.len(), "Kanji list retrieved");
        Ok(result)
    }
}
