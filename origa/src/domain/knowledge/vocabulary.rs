use crate::domain::dictionary::{KanjiInfo, get_kanji_info};
use crate::domain::japanese::JapaneseChar;
use crate::domain::tokenizer::{PartOfSpeech, tokenize_text};
use crate::domain::{Answer, JapaneseLevel, NativeLanguage, Question};
use crate::domain::{GrammarRule, OrigaError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyCard {
    word: Question,
    meaning: Answer,
}

impl VocabularyCard {
    pub fn new(word: Question, meaning: Answer) -> Self {
        Self { word, meaning }
    }

    pub fn word(&self) -> &Question {
        &self.word
    }

    pub fn meaning(&self) -> &Answer {
        &self.meaning
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
            meaning: self.meaning.clone(),
        };

        Ok((card, grammar_description))
    }

    pub fn revert(&self) -> Result<Self, OrigaError> {
        Ok(Self {
            word: Question::new(self.meaning.text().to_string())?,
            meaning: Answer::new(self.word.text().to_string())?,
        })
    }
}
