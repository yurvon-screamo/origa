use crate::dictionary::{KanjiInfo, get_kanji_info, get_translation};
use crate::domain::GrammarRule;
use crate::domain::japanese::JapaneseChar;
use crate::domain::tokenizer::{PartOfSpeech, tokenize_text};
use crate::domain::{Answer, JapaneseLevel, NativeLanguage, OrigaError, Question};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyCard {
    word: Question,
    reverse_side: Option<Question>,
}

impl VocabularyCard {
    pub fn new(word: Question) -> Self {
        Self {
            word,
            reverse_side: None,
        }
    }

    pub fn validate_translation(word: &str, lang: &NativeLanguage) -> Result<String, OrigaError> {
        let translation = get_translation(word, lang);
        match translation {
            Some(t) if !t.is_empty() => Ok(t),
            _ => Err(OrigaError::VocabularyNotFound {
                word: word.to_string(),
            }),
        }
    }

    pub fn word(&self) -> &Question {
        &self.word
    }

    pub fn question(&self) -> Question {
        self.word.clone()
    }

    pub fn answer(&self, lang: &NativeLanguage) -> Result<Answer, OrigaError> {
        if let Some(ref original) = self.reverse_side {
            return Answer::new(original.text().to_string()).map_err(|e| {
                OrigaError::InvalidAnswer {
                    reason: e.to_string(),
                }
            });
        }

        get_translation(self.word.text(), lang)
            .ok_or_else(|| OrigaError::TranslationNotFound {
                word: self.word.text().to_string(),
                lang: *lang,
            })
            .and_then(|t| {
                Answer::new(t).map_err(|e| OrigaError::InvalidAnswer {
                    reason: e.to_string(),
                })
            })
    }

    pub fn get_kanji_cards(&self, current_level: &JapaneseLevel) -> Vec<&KanjiInfo> {
        self.word
            .text()
            .chars()
            .filter(|c| c.is_kanji())
            .filter_map(|c| get_kanji_info(&c.to_string()).ok())
            .filter(|k: &&KanjiInfo| k.jlpt() <= current_level)
            .collect::<Vec<_>>()
    }

    pub fn part_of_speech(&self) -> Result<PartOfSpeech, OrigaError> {
        let tokens = tokenize_text(self.word.text())?;
        let token = tokens.first().ok_or(OrigaError::TokenizerError {
            reason: "Not found token".to_string(),
        })?;
        Ok(token.part_of_speech().clone())
    }

    pub fn with_grammar_rule(
        &self,
        rule: &GrammarRule,
        lang: &NativeLanguage,
    ) -> Result<(Self, String), OrigaError> {
        let formatted_word = rule.format(self.word.text(), &self.part_of_speech()?)?;
        let grammar_description = rule.content(lang).short_description().to_string();

        let answer_text = Self::validate_translation(self.word.text(), lang).and_then(|t| {
            Question::new(t).map_err(|e| OrigaError::InvalidQuestion {
                reason: e.to_string(),
            })
        })?;

        let card = Self {
            word: Question::new(formatted_word)?,
            reverse_side: Some(answer_text),
        };

        Ok((card, grammar_description))
    }

    pub fn revert(&self, lang: &NativeLanguage) -> Result<Self, OrigaError> {
        let meaning_text = self.answer(lang)?.text().to_string();
        Ok(Self {
            word: Question::new(meaning_text)?,
            reverse_side: Some(self.word.clone()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::Question;
    use crate::use_cases::init_real_dictionaries;

    fn create_vocab_card(word: &str) -> VocabularyCard {
        VocabularyCard::new(Question::new(word.to_string()).unwrap())
    }

    #[test]
    fn new_creates_card_with_word() {
        let question = Question::new("猫".to_string()).unwrap();
        let card = VocabularyCard::new(question.clone());

        assert_eq!(card.word(), &question);
        assert_eq!(card.word().text(), "猫");
    }

    #[test]
    fn new_creates_card_without_reverse_side() {
        let card = create_vocab_card("犬");

        assert!(card.word().text() == "犬");
    }

    #[test]
    fn word_returns_reference_to_question() {
        let card = create_vocab_card("猫");

        let word = card.word();

        assert_eq!(word.text(), "猫");
    }

    #[test]
    fn question_returns_cloned_question() {
        let card = create_vocab_card("猫");

        let question = card.question();

        assert_eq!(question.text(), "猫");
        assert_ne!(std::ptr::addr_of!(card.word), std::ptr::addr_of!(question));
    }

    #[test]
    fn answer_returns_translation_from_dictionary() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");
        let lang = NativeLanguage::Russian;

        let answer = card.answer(&lang);

        assert!(answer.is_ok());
        let binding = answer.unwrap();
        let answer_text = binding.text();
        assert!(answer_text.contains("кошка") || answer_text.contains("кот"));
    }

    #[test]
    fn answer_returns_error_for_unknown_word() {
        init_real_dictionaries();
        let card = create_vocab_card("存在しない言葉");
        let lang = NativeLanguage::Russian;

        let result = card.answer(&lang);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OrigaError::TranslationNotFound { .. }
        ));
    }

    #[test]
    fn answer_returns_reverse_side_when_present() {
        let question = Question::new("猫".to_string()).unwrap();
        let reverse_side = Question::new("кошка".to_string()).unwrap();
        let card = VocabularyCard {
            word: question,
            reverse_side: Some(reverse_side),
        };

        let answer = card.answer(&NativeLanguage::Russian);

        assert!(answer.is_ok());
        assert_eq!(answer.unwrap().text(), "кошка");
    }

    #[test]
    fn answer_returns_different_translations_for_different_languages() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");

        let russian_binding = card.answer(&NativeLanguage::Russian).unwrap();
        let russian_text = russian_binding.text().to_string();
        let english_binding = card.answer(&NativeLanguage::English).unwrap();
        let english_text = english_binding.text().to_string();

        assert_ne!(russian_text, english_text);
    }

    #[test]
    fn validate_translation_returns_translation_for_known_word() {
        init_real_dictionaries();

        let result = VocabularyCard::validate_translation("猫", &NativeLanguage::Russian);

        assert!(result.is_ok());
        let translation = result.unwrap();
        assert!(translation.contains("кошка") || translation.contains("кот"));
    }

    #[test]
    fn validate_translation_returns_error_for_unknown_word() {
        init_real_dictionaries();

        let result =
            VocabularyCard::validate_translation("存在しない言葉", &NativeLanguage::Russian);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OrigaError::VocabularyNotFound { .. }
        ));
    }

    #[test]
    fn validate_translation_returns_error_for_empty_translation() {
        init_real_dictionaries();

        let result = VocabularyCard::validate_translation("", &NativeLanguage::Russian);

        assert!(result.is_err());
    }

    #[test]
    fn get_kanji_cards_returns_empty_for_hiragana() {
        init_real_dictionaries();
        let card = create_vocab_card("ねこ");
        let level = JapaneseLevel::N5;

        let kanji_cards = card.get_kanji_cards(&level);

        assert!(kanji_cards.is_empty());
    }

    #[test]
    fn get_kanji_cards_returns_empty_for_katakana() {
        init_real_dictionaries();
        let card = create_vocab_card("ネコ");
        let level = JapaneseLevel::N5;

        let kanji_cards = card.get_kanji_cards(&level);

        assert!(kanji_cards.is_empty());
    }

    #[test]
    fn get_kanji_cards_returns_kanji_for_kanji_word() {
        init_real_dictionaries();
        let card = create_vocab_card("日本語");
        let level = JapaneseLevel::N1;

        let kanji_cards = card.get_kanji_cards(&level);

        assert!(!kanji_cards.is_empty());
    }

    #[test]
    fn get_kanji_cards_filters_by_level() {
        init_real_dictionaries();
        let card = create_vocab_card("日本語");
        let level_n5 = JapaneseLevel::N5;

        let kanji_cards = card.get_kanji_cards(&level_n5);

        assert!(kanji_cards.iter().all(|k| k.jlpt() <= &level_n5));
    }

    #[test]
    fn get_kanji_cards_returns_multiple_kanji() {
        init_real_dictionaries();
        let card = create_vocab_card("日本語");
        let level = JapaneseLevel::N1;

        let kanji_cards = card.get_kanji_cards(&level);

        assert!(kanji_cards.len() >= 2);
    }

    #[test]
    fn part_of_speech_returns_noun_for_noun() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");

        let pos = card.part_of_speech();

        assert!(pos.is_ok());
    }

    #[test]
    fn part_of_speech_returns_verb_for_verb() {
        init_real_dictionaries();
        let card = create_vocab_card("食べる");

        let pos = card.part_of_speech();

        assert!(pos.is_ok());
    }

    #[test]
    fn part_of_speech_returns_error_for_single_space() {
        init_real_dictionaries();
        let question = Question::new(" ".to_string());

        assert!(question.is_err());
    }

    #[test]
    fn revert_creates_reversed_card() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");
        let lang = NativeLanguage::Russian;

        let reversed = card.revert(&lang);

        assert!(reversed.is_ok());
        let reversed_card = reversed.unwrap();
        assert!(
            reversed_card.word().text().contains("кошка")
                || reversed_card.word().text().contains("кот")
        );
    }

    #[test]
    fn with_grammar_rule_returns_error_for_invalid_rule() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");
        let lang = NativeLanguage::Russian;

        let rule_id = ulid::Ulid::from_string("01KJ9AVWBGW9JTQNV2RXVNVWXR").expect("Invalid ULID");
        let rule = crate::domain::get_rule_by_id(&rule_id).expect("Rule not found");
        let result = card.with_grammar_rule(rule, &lang);

        assert!(result.is_err());
    }

    #[test]
    fn with_grammar_rule_stores_translation_as_reverse_side() {
        init_real_dictionaries();
        let card = create_vocab_card("食べる");
        let lang = NativeLanguage::Russian;

        let rule_id = ulid::Ulid::from_string("01KJ9AVWBGW9JTQNV2RXVNVWXR").expect("Invalid ULID");
        let rule = crate::domain::get_rule_by_id(&rule_id).expect("Rule not found");
        let result = card.with_grammar_rule(rule, &lang);

        assert!(result.is_ok());
        let (mutated_card, _) = result.unwrap();
        let answer = mutated_card.answer(&lang).unwrap();
        assert!(answer.text().contains("есть") || answer.text().contains("кушать"));
    }

    #[test]
    fn serialization_roundtrip() {
        let card = create_vocab_card("猫");

        let json = serde_json::to_string(&card).unwrap();
        let deserialized: VocabularyCard = serde_json::from_str(&json).unwrap();

        assert_eq!(card, deserialized);
    }

    #[test]
    fn serialization_roundtrip_with_reverse_side() {
        let question = Question::new("猫".to_string()).unwrap();
        let reverse_side = Question::new("кошка".to_string()).unwrap();
        let card = VocabularyCard {
            word: question,
            reverse_side: Some(reverse_side),
        };

        let json = serde_json::to_string(&card).unwrap();
        let deserialized: VocabularyCard = serde_json::from_str(&json).unwrap();

        assert_eq!(card, deserialized);
    }

    #[test]
    fn serialization_contains_expected_fields() {
        let card = create_vocab_card("猫");

        let json = serde_json::to_string(&card).unwrap();

        assert!(json.contains("猫"));
        assert!(json.contains("word"));
    }

    #[test]
    fn clone_creates_equal_copy() {
        let card = create_vocab_card("猫");

        let cloned = card.clone();

        assert_eq!(card, cloned);
    }

    #[test]
    fn debug_format_contains_word() {
        let card = create_vocab_card("猫");

        let debug_output = format!("{:?}", card);

        assert!(debug_output.contains("猫"));
    }
}
