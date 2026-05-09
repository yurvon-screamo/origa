use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::value_objects::NativeLanguage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum OrigaError {
    #[error("Current user does not exist")]
    CurrentUserNotExist,
    #[error("Card with id {card_id} not found")]
    CardNotFound { card_id: Ulid },
    #[error("Card with question '{question}' already exists")]
    DuplicateCard { question: String },
    #[error("Invalid question: {reason}")]
    InvalidQuestion { reason: String },
    #[error("Invalid answer: {reason}")]
    InvalidAnswer { reason: String },
    #[error("Invalid stability: {reason}")]
    InvalidStability { reason: String },
    #[error("Invalid difficulty: {reason}")]
    InvalidDifficulty { reason: String },
    #[error("Invalid memory state: {reason}")]
    InvalidMemoryState { reason: String },
    #[error("SRS calculation failed: {reason}")]
    SrsCalculationFailed { reason: String },
    #[error("Repository error: {reason}")]
    RepositoryError { reason: String },
    #[error("Embedding error: {reason}")]
    EmbeddingError { reason: String },
    #[error("LLM error: {reason}")]
    LlmError { reason: String },
    #[error("Settings error: {reason}")]
    SettingsError { reason: String },
    #[error("Furigana error: {reason}")]
    FuriganaError { reason: String },
    #[error("Translation error: {reason}")]
    TranslationError { reason: String },
    #[error("Kradfile error: {reason}")]
    KradfileError { reason: String },
    #[error("Vocabulary parse error: {reason}")]
    VocabularyParseError { reason: String },
    #[error("Invalid values: {reason}")]
    InvalidValues { reason: String },
    #[error("Tokenizer error: {reason}")]
    TokenizerError { reason: String },
    #[error("Grammar rule format error: {reason}")]
    GrammarFormatError { reason: String },
    #[error("Grammar parse error: {reason}")]
    GrammarParseError { reason: String },
    #[error("Well-known set parse error: {reason}")]
    WellKnownSetParseError { reason: String },
    #[error("Well-known set '{set_id}' not found")]
    WellKnownSetNotFound { set_id: String },
    #[error("Session expired, please login again")]
    SessionExpired,
    #[error("Dictionary not found: {reason}")]
    DictionaryNotFound { reason: String },
    #[error("Translation not found for word: {word}")]
    VocabularyNotFound { word: String },
    #[error("OCR error: {reason}")]
    OcrError { reason: String },
    #[error("Speech-to-text error: {reason}")]
    SttError { reason: String },
    #[error("Invalid Anki file: {reason}")]
    AnkiInvalidFile { reason: String },
    #[error("Database '{filename}' not found in Anki archive")]
    AnkiDatabaseNotFound { filename: String },
    #[error("Field '{field_name}' not found in Anki deck models")]
    AnkiFieldNotFound { field_name: String },
    #[error("No description for kanji: {kanji}")]
    KanjiNotFound { kanji: String },
    #[error("Grammar rule not found: {rule_id}")]
    GrammarRuleNotFound { rule_id: Ulid },
    #[error("Grammar rule {rule_id} content not found for language {lang}")]
    GrammarContentNotFound { rule_id: Ulid, lang: NativeLanguage },
    #[error("No translation for: {word} ({lang})")]
    TranslationNotFound { word: String, lang: NativeLanguage },
    #[error("Failed to delete account: {reason}")]
    AccountDeletionFailed { reason: String },
    #[error("Network error fetching {url}: {reason}")]
    NetworkError { url: String, reason: String },
    #[error("Phrase parse error: {reason}")]
    PhraseParseError { reason: String },
    #[error("Phrase not found: {phrase_id}")]
    PhraseNotFound { phrase_id: Ulid },
    #[error("Pitch audio parse error: {reason}")]
    PitchAudioParseError { reason: String },
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
        let error = OrigaError::CurrentUserNotExist;
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
        assert_display_contains(&error, "Well-known set parse error");
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
    fn phrase_not_found() {
        let phrase_id = Ulid::new();
        let error = OrigaError::PhraseNotFound { phrase_id };
        assert_display_contains(&error, &phrase_id.to_string());
        assert_serialization_roundtrip(error);
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

    #[test]
    fn pitch_audio_parse_error() {
        let error = OrigaError::PitchAudioParseError {
            reason: "invalid format".into(),
        };
        assert_display_contains(&error, "Pitch audio parse error");
        assert_serialization_roundtrip(error);
    }
}
