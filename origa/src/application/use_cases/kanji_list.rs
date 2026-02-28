use crate::domain::{get_kanji_list, JapaneseLevel, KanjiInfo, OrigaError};

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
        Ok(get_kanji_list(level)
            .into_iter()
            .map(|x| (*x).clone())
            .collect())
    }
}
