use crate::domain::JapaneseLevel;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct JlptContent {
    pub kanji_by_level: HashMap<JapaneseLevel, HashSet<String>>,
    pub words_by_level: HashMap<JapaneseLevel, HashSet<String>>,
    pub grammar_by_level: HashMap<JapaneseLevel, HashSet<String>>,
}

impl JlptContent {
    pub fn new() -> Self {
        Self {
            kanji_by_level: HashMap::new(),
            words_by_level: HashMap::new(),
            grammar_by_level: HashMap::new(),
        }
    }

    pub fn total_kanji(&self, level: JapaneseLevel) -> usize {
        self.kanji_by_level.get(&level).map_or(0, HashSet::len)
    }

    pub fn total_words(&self, level: JapaneseLevel) -> usize {
        self.words_by_level.get(&level).map_or(0, HashSet::len)
    }

    pub fn total_grammar(&self, level: JapaneseLevel) -> usize {
        self.grammar_by_level.get(&level).map_or(0, HashSet::len)
    }
}

impl Default for JlptContent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JlptContentError {
    #[error("Failed to load kanji data: {0}")]
    KanjiLoadError(String),

    #[error("Failed to load words data for level {0}: {1}")]
    WordsLoadError(String, String),

    #[error("Failed to load grammar data: {0}")]
    GrammarLoadError(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
