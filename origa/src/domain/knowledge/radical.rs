use crate::dictionary::radical::{RadicalInfo, get_radical_info};
use crate::domain::{OrigaError, Question};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadicalCard {
    radical: Question,
}

impl RadicalCard {
    pub fn new(radical: char) -> Result<Self, OrigaError> {
        get_radical_info(radical)?;
        Ok(Self {
            radical: Question::new(radical.to_string())?,
        })
    }

    pub fn radical_char(&self) -> char {
        self.radical.text().chars().next().unwrap()
    }

    pub fn radical_info(&self) -> Result<&'static RadicalInfo, OrigaError> {
        get_radical_info(self.radical_char())
    }

    pub fn kanji_examples(&self) -> Vec<char> {
        self.radical_info()
            .map(|info| info.kanji().to_vec())
            .unwrap_or_default()
    }

    pub fn question(&self) -> &Question {
        &self.radical
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::init_real_dictionaries;

    fn setup() {
        init_real_dictionaries();
    }

    #[test]
    fn new_creates_radical_card_for_valid_radical() {
        setup();
        let result = RadicalCard::new('一');
        assert!(result.is_ok());
        let card = result.unwrap();
        assert_eq!(card.radical_char(), '一');
    }

    #[test]
    fn new_fails_for_invalid_radical() {
        let result = RadicalCard::new('あ');
        assert!(result.is_err());
    }

    #[test]
    fn radical_info_returns_info() {
        setup();
        let card = RadicalCard::new('一').unwrap();
        let info = card.radical_info();
        assert!(info.is_ok());
    }

    #[test]
    fn kanji_examples_returns_kanji_list() {
        setup();
        let card = RadicalCard::new('一').unwrap();
        let examples = card.kanji_examples();
        assert!(!examples.is_empty());
    }

    #[test]
    fn serialization_roundtrip() {
        setup();
        let original = RadicalCard::new('一').unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: RadicalCard = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }
}
