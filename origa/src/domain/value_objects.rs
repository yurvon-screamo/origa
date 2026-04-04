use crate::domain::OrigaError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    text: String,
}

impl Question {
    pub fn new(text: String) -> Result<Self, OrigaError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(OrigaError::InvalidQuestion {
                reason: "Question text cannot be empty".to_string(),
            });
        }

        Ok(Self {
            text: text.to_string(),
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    text: String,
}

impl Answer {
    pub fn new(text: String) -> Result<Self, OrigaError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(OrigaError::InvalidAnswer {
                reason: "Answer text cannot be empty".to_string(),
            });
        }

        Ok(Self {
            text: text.to_string(),
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum JapaneseLevel {
    N5,
    N4,
    N3,
    N2,
    N1,
}

impl JapaneseLevel {
    pub const ALL: [JapaneseLevel; 5] = [Self::N5, Self::N4, Self::N3, Self::N2, Self::N1];

    pub fn as_number(&self) -> u8 {
        match self {
            JapaneseLevel::N5 => 5,
            JapaneseLevel::N4 => 4,
            JapaneseLevel::N3 => 3,
            JapaneseLevel::N2 => 2,
            JapaneseLevel::N1 => 1,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            JapaneseLevel::N5 => "N5",
            JapaneseLevel::N4 => "N4",
            JapaneseLevel::N3 => "N3",
            JapaneseLevel::N2 => "N2",
            JapaneseLevel::N1 => "N1",
        }
    }

    pub fn from_str_or_default(s: &str) -> Self {
        s.parse().unwrap_or(JapaneseLevel::N1)
    }
}

impl fmt::Display for JapaneseLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_number())
    }
}

impl FromStr for JapaneseLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "N5" => Ok(JapaneseLevel::N5),
            "N4" => Ok(JapaneseLevel::N4),
            "N3" => Ok(JapaneseLevel::N3),
            "N2" => Ok(JapaneseLevel::N2),
            "N1" => Ok(JapaneseLevel::N1),
            other => Err(format!("Unknown Japanese level: {}", other)),
        }
    }
}

impl From<i32> for JapaneseLevel {
    fn from(value: i32) -> Self {
        match value {
            1 => JapaneseLevel::N1,
            2 => JapaneseLevel::N2,
            3 => JapaneseLevel::N3,
            4 => JapaneseLevel::N4,
            _ => JapaneseLevel::N5,
        }
    }
}

impl From<JapaneseLevel> for i32 {
    fn from(level: JapaneseLevel) -> Self {
        match level {
            JapaneseLevel::N1 => 1,
            JapaneseLevel::N2 => 2,
            JapaneseLevel::N3 => 3,
            JapaneseLevel::N4 => 4,
            JapaneseLevel::N5 => 5,
        }
    }
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeLanguage {
    English,
    Russian,
}

impl NativeLanguage {
    pub fn as_str(&self) -> &str {
        match self {
            NativeLanguage::English => "English",
            NativeLanguage::Russian => "Russian",
        }
    }
}

impl fmt::Display for NativeLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<i32> for NativeLanguage {
    fn from(value: i32) -> Self {
        match value {
            0 => NativeLanguage::English,
            _ => NativeLanguage::Russian,
        }
    }
}

impl From<NativeLanguage> for i32 {
    fn from(lang: NativeLanguage) -> Self {
        match lang {
            NativeLanguage::English => 0,
            NativeLanguage::Russian => 1,
        }
    }
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DailyLoad {
    Light,
    #[default]
    Medium,
    Hard,
    Impossible,
}

impl DailyLoad {
    pub fn new_cards_per_day(&self) -> usize {
        match self {
            DailyLoad::Light => 5,
            DailyLoad::Medium => 10,
            DailyLoad::Hard => 15,
            DailyLoad::Impossible => 25,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DailyLoad::Light => "Лёгкий",
            DailyLoad::Medium => "Средний",
            DailyLoad::Hard => "Сложный",
            DailyLoad::Impossible => "Невозможный",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            DailyLoad::Light => "Несколько минут в день, лёгкая нагрузка для комфортного старта",
            DailyLoad::Medium => "Около 15 минут в день, сбалансированный темп обучения",
            DailyLoad::Hard => "Около 30 минут в день, интенсивное обучение для быстрого прогресса",
            DailyLoad::Impossible => "45+ минут в день, максимальная нагрузка для самых упорных",
        }
    }

    pub fn all() -> &'static [DailyLoad] {
        &[
            DailyLoad::Light,
            DailyLoad::Medium,
            DailyLoad::Hard,
            DailyLoad::Impossible,
        ]
    }
}

impl From<i32> for DailyLoad {
    fn from(value: i32) -> Self {
        match value {
            0 => DailyLoad::Light,
            1 => DailyLoad::Medium,
            2 => DailyLoad::Hard,
            _ => DailyLoad::Impossible,
        }
    }
}

impl From<DailyLoad> for i32 {
    fn from(val: DailyLoad) -> Self {
        match val {
            DailyLoad::Light => 0,
            DailyLoad::Medium => 1,
            DailyLoad::Hard => 2,
            DailyLoad::Impossible => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // Question tests
    #[test]
    fn question_new_success() {
        let question = Question::new("valid text".to_string()).unwrap();
        assert_eq!(question.text(), "valid text");
    }

    #[test]
    fn question_new_trims_whitespace() {
        let question = Question::new("  trimmed  ".to_string()).unwrap();
        assert_eq!(question.text(), "trimmed");
    }

    #[test]
    fn question_new_empty_string_error() {
        let result = Question::new("".to_string());
        assert!(matches!(result, Err(OrigaError::InvalidQuestion { .. })));
    }

    #[test]
    fn question_new_whitespace_only_error() {
        let result = Question::new("   ".to_string());
        assert!(matches!(result, Err(OrigaError::InvalidQuestion { .. })));
    }

    // Answer tests
    #[test]
    fn answer_new_success() {
        let answer = Answer::new("valid text".to_string()).unwrap();
        assert_eq!(answer.text(), "valid text");
    }

    #[test]
    fn answer_new_trims_whitespace() {
        let answer = Answer::new("  trimmed  ".to_string()).unwrap();
        assert_eq!(answer.text(), "trimmed");
    }

    #[test]
    fn answer_new_empty_string_error() {
        let result = Answer::new("".to_string());
        assert!(matches!(result, Err(OrigaError::InvalidAnswer { .. })));
    }

    #[test]
    fn answer_new_whitespace_only_error() {
        let result = Answer::new("   ".to_string());
        assert!(matches!(result, Err(OrigaError::InvalidAnswer { .. })));
    }

    // JapaneseLevel::from_str tests
    #[rstest]
    #[case("N5", Ok(JapaneseLevel::N5))]
    #[case("N4", Ok(JapaneseLevel::N4))]
    #[case("N3", Ok(JapaneseLevel::N3))]
    #[case("N2", Ok(JapaneseLevel::N2))]
    #[case("N1", Ok(JapaneseLevel::N1))]
    fn japanese_level_from_str_success(
        #[case] input: &str,
        #[case] expected: Result<JapaneseLevel, String>,
    ) {
        let result: Result<JapaneseLevel, String> = input.parse();
        assert_eq!(result, expected);
    }

    #[test]
    fn japanese_level_from_str_lowercase_success() {
        let result: Result<JapaneseLevel, String> = "n5".parse();
        assert_eq!(result, Ok(JapaneseLevel::N5));
    }

    #[test]
    fn japanese_level_from_str_invalid_error() {
        let result: Result<JapaneseLevel, String> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn japanese_level_from_str_empty_error() {
        let result: Result<JapaneseLevel, String> = "".parse();
        assert!(result.is_err());
    }

    #[test]
    fn japanese_level_from_str_trims_whitespace() {
        let result: Result<JapaneseLevel, String> = "  N5  ".parse();
        assert_eq!(result, Ok(JapaneseLevel::N5));
    }

    // JapaneseLevel::from(i32) tests
    #[rstest]
    #[case(5, JapaneseLevel::N5)]
    #[case(4, JapaneseLevel::N4)]
    #[case(3, JapaneseLevel::N3)]
    #[case(2, JapaneseLevel::N2)]
    #[case(1, JapaneseLevel::N1)]
    fn japanese_level_from_i32_success(#[case] input: i32, #[case] expected: JapaneseLevel) {
        let result = JapaneseLevel::from(input);
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(0, JapaneseLevel::N5)]
    #[case(-1, JapaneseLevel::N5)]
    #[case(99, JapaneseLevel::N5)]
    fn japanese_level_from_i32_fallback(#[case] input: i32, #[case] expected: JapaneseLevel) {
        let result = JapaneseLevel::from(input);
        assert_eq!(result, expected);
    }

    // JapaneseLevel::as_number tests
    #[rstest]
    #[case(JapaneseLevel::N5, 5)]
    #[case(JapaneseLevel::N4, 4)]
    #[case(JapaneseLevel::N3, 3)]
    #[case(JapaneseLevel::N2, 2)]
    #[case(JapaneseLevel::N1, 1)]
    fn japanese_level_as_number(#[case] level: JapaneseLevel, #[case] expected: u8) {
        assert_eq!(level.as_number(), expected);
    }

    // JapaneseLevel::code tests
    #[rstest]
    #[case(JapaneseLevel::N5, "N5")]
    #[case(JapaneseLevel::N4, "N4")]
    #[case(JapaneseLevel::N3, "N3")]
    #[case(JapaneseLevel::N2, "N2")]
    #[case(JapaneseLevel::N1, "N1")]
    fn japanese_level_code(#[case] level: JapaneseLevel, #[case] expected: &str) {
        assert_eq!(level.code(), expected);
    }

    // JapaneseLevel::from_str_or_default tests
    #[rstest]
    #[case("N5", JapaneseLevel::N5)]
    #[case("N4", JapaneseLevel::N4)]
    #[case("N3", JapaneseLevel::N3)]
    #[case("N2", JapaneseLevel::N2)]
    #[case("N1", JapaneseLevel::N1)]
    fn japanese_level_from_str_or_default_success(
        #[case] input: &str,
        #[case] expected: JapaneseLevel,
    ) {
        let result = JapaneseLevel::from_str_or_default(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn japanese_level_from_str_or_default_fallback() {
        let result = JapaneseLevel::from_str_or_default("invalid");
        assert_eq!(result, JapaneseLevel::N1);
    }

    // JapaneseLevel Display trait test
    #[rstest]
    #[case(JapaneseLevel::N5, "5")]
    #[case(JapaneseLevel::N4, "4")]
    #[case(JapaneseLevel::N3, "3")]
    #[case(JapaneseLevel::N2, "2")]
    #[case(JapaneseLevel::N1, "1")]
    fn japanese_level_display(#[case] level: JapaneseLevel, #[case] expected: &str) {
        assert_eq!(format!("{}", level), expected);
    }

    // JapaneseLevel ALL constant test
    #[test]
    fn japanese_level_all_contains_all_levels() {
        assert_eq!(JapaneseLevel::ALL.len(), 5);
        assert!(JapaneseLevel::ALL.contains(&JapaneseLevel::N5));
        assert!(JapaneseLevel::ALL.contains(&JapaneseLevel::N4));
        assert!(JapaneseLevel::ALL.contains(&JapaneseLevel::N3));
        assert!(JapaneseLevel::ALL.contains(&JapaneseLevel::N2));
        assert!(JapaneseLevel::ALL.contains(&JapaneseLevel::N1));
    }

    // NativeLanguage::from(i32) tests
    #[rstest]
    #[case(0, NativeLanguage::English)]
    #[case(1, NativeLanguage::Russian)]
    fn native_language_from_i32_success(#[case] input: i32, #[case] expected: NativeLanguage) {
        let result = NativeLanguage::from(input);
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(-1, NativeLanguage::Russian)]
    #[case(99, NativeLanguage::Russian)]
    fn native_language_from_i32_fallback(#[case] input: i32, #[case] expected: NativeLanguage) {
        let result = NativeLanguage::from(input);
        assert_eq!(result, expected);
    }

    // NativeLanguage::as_str tests
    #[rstest]
    #[case(NativeLanguage::English, "English")]
    #[case(NativeLanguage::Russian, "Russian")]
    fn native_language_as_str(#[case] lang: NativeLanguage, #[case] expected: &str) {
        assert_eq!(lang.as_str(), expected);
    }

    // NativeLanguage Display trait test
    #[rstest]
    #[case(NativeLanguage::English, "English")]
    #[case(NativeLanguage::Russian, "Russian")]
    fn native_language_display(#[case] lang: NativeLanguage, #[case] expected: &str) {
        assert_eq!(format!("{}", lang), expected);
    }

    // Conversion from JapaneseLevel to i32
    #[rstest]
    #[case(JapaneseLevel::N1, 1)]
    #[case(JapaneseLevel::N2, 2)]
    #[case(JapaneseLevel::N3, 3)]
    #[case(JapaneseLevel::N4, 4)]
    #[case(JapaneseLevel::N5, 5)]
    fn japanese_level_into_i32(#[case] level: JapaneseLevel, #[case] expected: i32) {
        let result: i32 = level.into();
        assert_eq!(result, expected);
    }

    // Conversion from NativeLanguage to i32
    #[rstest]
    #[case(NativeLanguage::English, 0)]
    #[case(NativeLanguage::Russian, 1)]
    fn native_language_into_i32(#[case] lang: NativeLanguage, #[case] expected: i32) {
        let result: i32 = lang.into();
        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod tests_daily_load {
    use super::*;

    #[test]
    fn default_is_medium() {
        assert_eq!(DailyLoad::default(), DailyLoad::Medium);
    }

    #[test]
    fn new_cards_per_day_values() {
        assert_eq!(DailyLoad::Light.new_cards_per_day(), 5);
        assert_eq!(DailyLoad::Medium.new_cards_per_day(), 10);
        assert_eq!(DailyLoad::Hard.new_cards_per_day(), 15);
        assert_eq!(DailyLoad::Impossible.new_cards_per_day(), 25);
    }

    #[test]
    fn from_i32_roundtrip() {
        for val in DailyLoad::all() {
            let i = i32::from(*val);
            assert_eq!(DailyLoad::from(i), *val);
        }
    }

    #[test]
    fn from_i32_unknown_falls_back_to_impossible() {
        assert_eq!(DailyLoad::from(999), DailyLoad::Impossible);
    }
}
