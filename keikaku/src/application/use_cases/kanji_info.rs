use crate::domain::{KANJI_DICTIONARY, KanjiInfo, KeikakuError};

pub struct KanjiInfoUseCase;

impl Default for KanjiInfoUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl KanjiInfoUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, kanji: &str) -> Result<KanjiInfo, KeikakuError> {
        Ok(KANJI_DICTIONARY.get_kanji_info(kanji)?.to_owned())
    }
}
