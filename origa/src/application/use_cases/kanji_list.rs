use crate::domain::{JapaneseLevel, KANJI_DICTIONARY, KanjiInfo, OrigaError};

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
        Ok(KANJI_DICTIONARY
            .get_kanji_list(level)
            .iter()
            .map(|x| (*x).clone())
            .collect())
    }
}
