use crate::domain::{CardType, JapaneseLevel};
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

    pub fn find_level(&self, content_key: &str, card_type: CardType) -> Option<JapaneseLevel> {
        let collection = match card_type {
            CardType::Kanji => &self.kanji_by_level,
            CardType::Vocabulary => &self.words_by_level,
            CardType::Grammar => &self.grammar_by_level,
            CardType::Radical => return None,
        };

        JapaneseLevel::ALL
            .iter()
            .find(|&&level| {
                collection
                    .get(&level)
                    .is_some_and(|set| set.contains(content_key))
            })
            .copied()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_content() -> JlptContent {
        let mut content = JlptContent::new();
        content.kanji_by_level.insert(
            JapaneseLevel::N5,
            ["日".to_string(), "月".to_string()].into_iter().collect(),
        );
        content.words_by_level.insert(
            JapaneseLevel::N5,
            ["食べる".to_string(), "飲む".to_string()]
                .into_iter()
                .collect(),
        );
        content.grammar_by_level.insert(
            JapaneseLevel::N5,
            ["〜です".to_string(), "〜ます".to_string()]
                .into_iter()
                .collect(),
        );
        content
    }

    #[test]
    fn new_creates_empty_content() {
        let content = JlptContent::new();

        assert!(content.kanji_by_level.is_empty());
        assert!(content.words_by_level.is_empty());
        assert!(content.grammar_by_level.is_empty());
    }

    #[test]
    fn default_creates_empty_content() {
        let content = JlptContent::default();

        assert!(content.kanji_by_level.is_empty());
        assert!(content.words_by_level.is_empty());
        assert!(content.grammar_by_level.is_empty());
    }

    #[test]
    fn total_kanji_returns_correct_count_for_existing_level() {
        let content = create_test_content();

        let count = content.total_kanji(JapaneseLevel::N5);

        assert_eq!(count, 2);
    }

    #[test]
    fn total_kanji_returns_zero_for_missing_level() {
        let content = create_test_content();

        let count = content.total_kanji(JapaneseLevel::N1);

        assert_eq!(count, 0);
    }

    #[test]
    fn total_words_returns_correct_count_for_existing_level() {
        let content = create_test_content();

        let count = content.total_words(JapaneseLevel::N5);

        assert_eq!(count, 2);
    }

    #[test]
    fn total_words_returns_zero_for_missing_level() {
        let content = create_test_content();

        let count = content.total_words(JapaneseLevel::N1);

        assert_eq!(count, 0);
    }

    #[test]
    fn total_grammar_returns_correct_count_for_existing_level() {
        let content = create_test_content();

        let count = content.total_grammar(JapaneseLevel::N5);

        assert_eq!(count, 2);
    }

    #[test]
    fn total_grammar_returns_zero_for_missing_level() {
        let content = create_test_content();

        let count = content.total_grammar(JapaneseLevel::N1);

        assert_eq!(count, 0);
    }

    #[test]
    fn jlpt_content_error_kanji_load_displays_correctly() {
        let error = JlptContentError::KanjiLoadError("test error".to_string());

        assert_eq!(error.to_string(), "Failed to load kanji data: test error");
    }

    #[test]
    fn jlpt_content_error_words_load_displays_correctly() {
        let error =
            JlptContentError::WordsLoadError("N5".to_string(), "file not found".to_string());

        assert_eq!(
            error.to_string(),
            "Failed to load words data for level N5: file not found"
        );
    }

    #[test]
    fn jlpt_content_error_grammar_load_displays_correctly() {
        let error = JlptContentError::GrammarLoadError("missing file".to_string());

        assert_eq!(
            error.to_string(),
            "Failed to load grammar data: missing file"
        );
    }
}
