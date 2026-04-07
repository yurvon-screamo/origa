use serde::{Deserialize, Serialize};
use std::fmt;
use ulid::Ulid;

use super::value_objects::NativeLanguage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrigaError {
    CurrentUserNotExist {},
    CardNotFound { card_id: Ulid },
    DuplicateCard { question: String },
    InvalidQuestion { reason: String },
    InvalidAnswer { reason: String },
    InvalidStability { reason: String },
    InvalidDifficulty { reason: String },
    InvalidMemoryState { reason: String },
    SrsCalculationFailed { reason: String },
    RepositoryError { reason: String },
    EmbeddingError { reason: String },
    LlmError { reason: String },
    SettingsError { reason: String },
    FuriganaError { reason: String },
    TranslationError { reason: String },
    KradfileError { reason: String },
    VocabularyParseError { reason: String },
    InvalidValues { reason: String },
    TokenizerError { reason: String },
    GrammarFormatError { reason: String },
    GrammarParseError { reason: String },
    WellKnownSetParseError { reason: String },
    WellKnownSetNotFound { set_id: String },
    SessionExpired,
    DictionaryNotFound { reason: String },
    VocabularyNotFound { word: String },
    OcrError { reason: String },
    AnkiInvalidFile { reason: String },
    AnkiDatabaseNotFound { filename: String },
    AnkiFieldNotFound { field_name: String },
    KanjiNotFound { kanji: String },
    RadicalNotFound { radical: char },
    GrammarRuleNotFound { rule_id: Ulid },
    GrammarContentNotFound { rule_id: Ulid, lang: NativeLanguage },
    TranslationNotFound { word: String, lang: NativeLanguage },
    AccountDeletionFailed { reason: String },
    NetworkError { url: String, reason: String },
}

impl fmt::Display for OrigaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrigaError::CurrentUserNotExist {} => {
                write!(f, "Current user does not exist")
            },
            OrigaError::CardNotFound { card_id } => {
                write!(f, "Card with id {} not found", card_id)
            },
            OrigaError::DuplicateCard { question } => {
                write!(f, "Card with question '{}' already exists", question)
            },
            OrigaError::InvalidQuestion { reason } => {
                write!(f, "Invalid question: {}", reason)
            },
            OrigaError::InvalidAnswer { reason } => {
                write!(f, "Invalid answer: {}", reason)
            },
            OrigaError::InvalidStability { reason } => {
                write!(f, "Invalid stability: {}", reason)
            },
            OrigaError::InvalidDifficulty { reason } => {
                write!(f, "Invalid difficulty: {}", reason)
            },
            OrigaError::InvalidMemoryState { reason } => {
                write!(f, "Invalid memory state: {}", reason)
            },
            OrigaError::SrsCalculationFailed { reason } => {
                write!(f, "SRS calculation failed: {}", reason)
            },
            OrigaError::RepositoryError { reason } => {
                write!(f, "Repository error: {}", reason)
            },
            OrigaError::EmbeddingError { reason } => {
                write!(f, "Embedding error: {}", reason)
            },
            OrigaError::LlmError { reason } => {
                write!(f, "LLM error: {}", reason)
            },
            OrigaError::SettingsError { reason } => {
                write!(f, "Settings error: {}", reason)
            },
            OrigaError::FuriganaError { reason } => {
                write!(f, "Furigana error: {}", reason)
            },
            OrigaError::TranslationError { reason } => {
                write!(f, "Translation error: {}", reason)
            },
            OrigaError::KradfileError { reason } => {
                write!(f, "Kradfile error: {}", reason)
            },
            OrigaError::VocabularyParseError { reason } => {
                write!(f, "Vocabulary parse error: {}", reason)
            },
            OrigaError::InvalidValues { reason } => {
                write!(f, "Invalid values: {}", reason)
            },
            OrigaError::TokenizerError { reason } => {
                write!(f, "Tokenizer error: {}", reason)
            },
            OrigaError::GrammarFormatError { reason } => {
                write!(f, "Grammar rule format error: {}", reason)
            },
            OrigaError::GrammarParseError { reason } => {
                write!(f, "Grammar parse error: {}", reason)
            },
            OrigaError::WellKnownSetParseError { reason } => {
                write!(f, "WellKnownSetError: {}", reason)
            },
            OrigaError::WellKnownSetNotFound { set_id } => {
                write!(f, "WellKnownSet '{}' not found", set_id)
            },
            OrigaError::SessionExpired => {
                write!(f, "Session expired, please login again")
            },
            OrigaError::DictionaryNotFound { reason } => {
                write!(f, "Dictionary not found: {}", reason)
            },
            OrigaError::VocabularyNotFound { word } => {
                write!(f, "Translation not found for word: {}", word)
            },
            OrigaError::OcrError { reason } => {
                write!(f, "OCR error: {}", reason)
            },
            OrigaError::AnkiInvalidFile { reason } => {
                write!(f, "Invalid Anki file: {}", reason)
            },
            OrigaError::AnkiDatabaseNotFound { filename } => {
                write!(f, "Database '{}' not found in Anki archive", filename)
            },
            OrigaError::AnkiFieldNotFound { field_name } => {
                write!(f, "Field '{}' not found in Anki deck models", field_name)
            },
            OrigaError::KanjiNotFound { kanji } => {
                write!(f, "Нет описания для кандзи: {}", kanji)
            },
            OrigaError::RadicalNotFound { radical } => {
                write!(f, "Радикал не найден: {}", radical)
            },
            OrigaError::GrammarRuleNotFound { rule_id } => {
                write!(f, "Правило грамматики не найдено: {}", rule_id)
            },
            OrigaError::GrammarContentNotFound { rule_id, lang } => {
                write!(
                    f,
                    "Контент правила {} не найден для языка {}",
                    rule_id, lang
                )
            },
            OrigaError::TranslationNotFound { word, lang } => {
                write!(f, "Нет перевода для: {} ({})", word, lang)
            },
            OrigaError::AccountDeletionFailed { reason } => {
                write!(f, "Failed to delete account: {}", reason)
            },
            OrigaError::NetworkError { url, reason } => {
                write!(f, "Network error fetching {}: {}", url, reason)
            },
        }
    }
}

impl std::error::Error for OrigaError {}

impl From<ort::Error> for OrigaError {
    fn from(error: ort::Error) -> Self {
        OrigaError::OcrError {
            reason: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_display_contains(error: &OrigaError, expected: &str) {
        let display = format!("{}", error);
        assert!(
            display.contains(expected),
            "Display '{}' should contain '{}'",
            display,
            expected
        );
    }

    fn assert_serialization_roundtrip(error: OrigaError) {
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: OrigaError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn current_user_not_exist() {
        let error = OrigaError::CurrentUserNotExist {};
        assert_display_contains(&error, "Current user does not exist");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn card_not_found() {
        let card_id = Ulid::new();
        let error = OrigaError::CardNotFound { card_id };
        assert_display_contains(&error, &card_id.to_string());
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn duplicate_card() {
        let question = "test question".to_string();
        let error = OrigaError::DuplicateCard {
            question: question.clone(),
        };
        assert_display_contains(&error, &question);
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn invalid_question() {
        let error = OrigaError::InvalidQuestion {
            reason: "empty".into(),
        };
        assert_display_contains(&error, "Invalid question");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn invalid_answer() {
        let error = OrigaError::InvalidAnswer {
            reason: "too long".into(),
        };
        assert_display_contains(&error, "Invalid answer");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn invalid_stability() {
        let error = OrigaError::InvalidStability {
            reason: "negative".into(),
        };
        assert_display_contains(&error, "Invalid stability");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn invalid_difficulty() {
        let error = OrigaError::InvalidDifficulty {
            reason: "out of range".into(),
        };
        assert_display_contains(&error, "Invalid difficulty");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn invalid_memory_state() {
        let error = OrigaError::InvalidMemoryState {
            reason: "corrupt".into(),
        };
        assert_display_contains(&error, "Invalid memory state");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn srs_calculation_failed() {
        let error = OrigaError::SrsCalculationFailed {
            reason: "overflow".into(),
        };
        assert_display_contains(&error, "SRS calculation failed");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn repository_error() {
        let error = OrigaError::RepositoryError {
            reason: "connection failed".into(),
        };
        assert_display_contains(&error, "Repository error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn embedding_error() {
        let error = OrigaError::EmbeddingError {
            reason: "model not loaded".into(),
        };
        assert_display_contains(&error, "Embedding error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn llm_error() {
        let error = OrigaError::LlmError {
            reason: "timeout".into(),
        };
        assert_display_contains(&error, "LLM error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn settings_error() {
        let error = OrigaError::SettingsError {
            reason: "invalid config".into(),
        };
        assert_display_contains(&error, "Settings error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn furigana_error() {
        let error = OrigaError::FuriganaError {
            reason: "parse failed".into(),
        };
        assert_display_contains(&error, "Furigana error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn translation_error() {
        let error = OrigaError::TranslationError {
            reason: "api error".into(),
        };
        assert_display_contains(&error, "Translation error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn kradfile_error() {
        let error = OrigaError::KradfileError {
            reason: "file not found".into(),
        };
        assert_display_contains(&error, "Kradfile error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn vocabulary_parse_error() {
        let error = OrigaError::VocabularyParseError {
            reason: "invalid format".into(),
        };
        assert_display_contains(&error, "Vocabulary parse error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn invalid_values() {
        let error = OrigaError::InvalidValues {
            reason: "missing fields".into(),
        };
        assert_display_contains(&error, "Invalid values");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn tokenizer_error() {
        let error = OrigaError::TokenizerError {
            reason: "dictionary missing".into(),
        };
        assert_display_contains(&error, "Tokenizer error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn grammar_format_error() {
        let error = OrigaError::GrammarFormatError {
            reason: "bad syntax".into(),
        };
        assert_display_contains(&error, "Grammar rule format error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn grammar_parse_error() {
        let error = OrigaError::GrammarParseError {
            reason: "unexpected token".into(),
        };
        assert_display_contains(&error, "Grammar parse error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn well_known_set_parse_error() {
        let error = OrigaError::WellKnownSetParseError {
            reason: "invalid id".into(),
        };
        assert_display_contains(&error, "WellKnownSetError");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn well_known_set_not_found() {
        let error = OrigaError::WellKnownSetNotFound {
            set_id: "test_set".into(),
        };
        assert_display_contains(&error, "test_set");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn session_expired() {
        let error = OrigaError::SessionExpired;
        assert_display_contains(&error, "Session expired");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn dictionary_not_found() {
        let error = OrigaError::DictionaryNotFound {
            reason: "jmdict".into(),
        };
        assert_display_contains(&error, "Dictionary not found");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn vocabulary_not_found() {
        let error = OrigaError::VocabularyNotFound {
            word: "日本語".into(),
        };
        assert_display_contains(&error, "日本語");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn ocr_error() {
        let error = OrigaError::OcrError {
            reason: "image corrupt".into(),
        };
        assert_display_contains(&error, "OCR error");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn kanji_not_found() {
        let error = OrigaError::KanjiNotFound {
            kanji: "日".into()
        };
        assert_display_contains(&error, "日");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn radical_not_found() {
        let error = OrigaError::RadicalNotFound { radical: '一' };
        assert_display_contains(&error, "一");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn grammar_rule_not_found() {
        let rule_id = Ulid::new();
        let error = OrigaError::GrammarRuleNotFound { rule_id };
        assert_display_contains(&error, &rule_id.to_string());
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn grammar_content_not_found() {
        let rule_id = Ulid::new();
        let error = OrigaError::GrammarContentNotFound {
            rule_id,
            lang: NativeLanguage::Russian,
        };
        assert_display_contains(&error, &rule_id.to_string());
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn translation_not_found() {
        let error = OrigaError::TranslationNotFound {
            word: "猫".into(),
            lang: NativeLanguage::Russian,
        };
        assert_display_contains(&error, "猫");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn network_error() {
        let error = OrigaError::NetworkError {
            url: "https://api.example.com".into(),
            reason: "timeout".into(),
        };
        assert_display_contains(&error, "https://api.example.com");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn account_deletion_failed() {
        let error = OrigaError::AccountDeletionFailed {
            reason: "server timeout".into(),
        };
        assert_display_contains(&error, "Failed to delete account");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn error_trait_implementation() {
        let error = OrigaError::CardNotFound {
            card_id: Ulid::new(),
        };
        let _: &dyn std::error::Error = &error;
    }

    #[test]
    fn anki_invalid_file() {
        let error = OrigaError::AnkiInvalidFile {
            reason: "not a zip".into(),
        };
        assert_display_contains(&error, "Invalid Anki file");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn anki_database_not_found() {
        let error = OrigaError::AnkiDatabaseNotFound {
            filename: "collection.anki21".into(),
        };
        assert_display_contains(&error, "collection.anki21");
        assert_serialization_roundtrip(error);
    }

    #[test]
    fn anki_field_not_found() {
        let error = OrigaError::AnkiFieldNotFound {
            field_name: "Expression".into(),
        };
        assert_display_contains(&error, "Expression");
        assert_serialization_roundtrip(error);
    }
}
