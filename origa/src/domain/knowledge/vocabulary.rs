use crate::domain::dictionary::{KanjiInfo, get_kanji_info, get_translation};
use crate::domain::japanese::JapaneseChar;
use crate::domain::tokenizer::{PartOfSpeech, tokenize_text};
use crate::domain::{Answer, FALLBACK_ANSWER, JapaneseLevel, NativeLanguage, Question};
use crate::domain::{GrammarRule, OrigaError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyCard {
    word: Question,
    // TODO: костыль
    original_word: Option<Question>,
}

impl VocabularyCard {
    pub fn new(word: Question) -> Self {
        Self {
            word,
            original_word: None,
        }
    }

    pub fn word(&self) -> &Question {
        &self.word
    }

    pub fn question(&self) -> Question {
        self.word.clone()
    }

    pub fn answer(&self, lang: &NativeLanguage) -> Answer {
        if let Some(ref original) = self.original_word {
            return Answer::new(original.text().to_string())
                .unwrap_or_else(|_| Answer::new(FALLBACK_ANSWER.to_string()).unwrap());
        }

        get_translation(self.word.text(), lang)
            .and_then(|t| Answer::new(t).ok())
            .unwrap_or_else(|| Answer::new(FALLBACK_ANSWER.to_string()).unwrap())
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

        let card = Self {
            word: Question::new(formatted_word)?,
            original_word: self.original_word.clone(),
        };

        Ok((card, grammar_description))
    }

    pub fn revert(&self, lang: &NativeLanguage) -> Result<Self, OrigaError> {
        let meaning_text = self.answer(lang).text().to_string();
        Ok(Self {
            word: Question::new(meaning_text)?,
            original_word: Some(self.word.clone()),
        })
    }
}
