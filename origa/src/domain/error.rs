use serde::{Deserialize, Serialize};
use std::fmt;
use ulid::Ulid;

use super::value_objects::NativeLanguage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrigaError {
    UserNotFound { user_id: Ulid },
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
    SessionExpired,
    DictionaryNotFound { reason: String },
    VocabularyNotFound { word: String },
    OcrError { reason: String },
    KanjiNotFound { kanji: String },
    GrammarRuleNotFound { rule_id: Ulid },
    GrammarContentNotFound { rule_id: Ulid, lang: NativeLanguage },
    TranslationNotFound { word: String, lang: NativeLanguage },
    NetworkError { url: String, reason: String },
}

impl fmt::Display for OrigaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrigaError::UserNotFound { user_id } => {
                write!(f, "User with id {} not found", user_id)
            }

            OrigaError::CardNotFound { card_id } => {
                write!(f, "Card with id {} not found", card_id)
            }
            OrigaError::DuplicateCard { question } => {
                write!(f, "Card with question '{}' already exists", question)
            }
            OrigaError::InvalidQuestion { reason } => {
                write!(f, "Invalid question: {}", reason)
            }
            OrigaError::InvalidAnswer { reason } => {
                write!(f, "Invalid answer: {}", reason)
            }
            OrigaError::InvalidStability { reason } => {
                write!(f, "Invalid stability: {}", reason)
            }
            OrigaError::InvalidDifficulty { reason } => {
                write!(f, "Invalid difficulty: {}", reason)
            }
            OrigaError::InvalidMemoryState { reason } => {
                write!(f, "Invalid memory state: {}", reason)
            }
            OrigaError::SrsCalculationFailed { reason } => {
                write!(f, "SRS calculation failed: {}", reason)
            }
            OrigaError::RepositoryError { reason } => {
                write!(f, "Repository error: {}", reason)
            }
            OrigaError::EmbeddingError { reason } => {
                write!(f, "Embedding error: {}", reason)
            }
            OrigaError::LlmError { reason } => {
                write!(f, "LLM error: {}", reason)
            }
            OrigaError::SettingsError { reason } => {
                write!(f, "Settings error: {}", reason)
            }
            OrigaError::FuriganaError { reason } => {
                write!(f, "Furigana error: {}", reason)
            }
            OrigaError::TranslationError { reason } => {
                write!(f, "Translation error: {}", reason)
            }
            OrigaError::KradfileError { reason } => {
                write!(f, "Kradfile error: {}", reason)
            }
            OrigaError::VocabularyParseError { reason } => {
                write!(f, "Vocabulary parse error: {}", reason)
            }
            OrigaError::InvalidValues { reason } => {
                write!(f, "Invalid values: {}", reason)
            }
            OrigaError::TokenizerError { reason } => {
                write!(f, "Tokenizer error: {}", reason)
            }
            OrigaError::GrammarFormatError { reason } => {
                write!(f, "Grammar rule format error: {}", reason)
            }
            OrigaError::GrammarParseError { reason } => {
                write!(f, "Grammar parse error: {}", reason)
            }
            OrigaError::WellKnownSetParseError { reason } => {
                write!(f, "WellKnownSetError: {}", reason)
            }
            OrigaError::SessionExpired => {
                write!(f, "Session expired, please login again")
            }
            OrigaError::DictionaryNotFound { reason } => {
                write!(f, "Dictionary not found: {}", reason)
            }
            OrigaError::VocabularyNotFound { word } => {
                write!(f, "Translation not found for word: {}", word)
            }
            OrigaError::OcrError { reason } => {
                write!(f, "OCR error: {}", reason)
            }
            OrigaError::KanjiNotFound { kanji } => {
                write!(f, "Нет описания для кандзи: {}", kanji)
            }
            OrigaError::GrammarRuleNotFound { rule_id } => {
                write!(f, "Правило грамматики не найдено: {}", rule_id)
            }
            OrigaError::GrammarContentNotFound { rule_id, lang } => {
                write!(
                    f,
                    "Контент правила {} не найден для языка {}",
                    rule_id, lang
                )
            }
            OrigaError::TranslationNotFound { word, lang } => {
                write!(f, "Нет перевода для: {} ({})", word, lang)
            }
            OrigaError::NetworkError { url, reason } => {
                write!(f, "Network error fetching {}: {}", url, reason)
            }
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
