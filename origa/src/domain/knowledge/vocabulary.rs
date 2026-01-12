use crate::domain::OrigaError;
use crate::domain::dictionary::{KANJI_DICTIONARY, KanjiInfo};
use crate::domain::grammar::GrammarRule;
use crate::domain::japanese::JapaneseChar;
use crate::domain::tokenizer::{PartOfSpeech, tokenize_text};
use crate::domain::{Answer, JapaneseLevel, NativeLanguage, Question};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyCard {
    word: Question,
    meaning: Answer,
    example_phrases: Vec<ExamplePhrase>,
}

impl VocabularyCard {
    pub fn new(word: Question, meaning: Answer, example_phrases: Vec<ExamplePhrase>) -> Self {
        Self {
            word,
            meaning,
            example_phrases,
        }
    }

    pub fn word(&self) -> &Question {
        &self.word
    }

    pub fn meaning(&self) -> &Answer {
        &self.meaning
    }

    pub fn example_phrases(&self) -> &[ExamplePhrase] {
        &self.example_phrases
    }

    pub fn get_kanji_cards(&self, current_level: &JapaneseLevel) -> Vec<&KanjiInfo> {
        self.word
            .text()
            .chars()
            .filter(|c| c.is_kanji())
            .filter_map(|c| KANJI_DICTIONARY.get_kanji_info(&c.to_string()).ok())
            .filter(|k| k.jlpt() <= current_level)
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
        rule: &dyn GrammarRule,
        lang: &NativeLanguage,
    ) -> Result<Self, OrigaError> {
        let formatted_word = rule.format(self.word.text(), &self.part_of_speech()?)?;
        let meaning = self.meaning.text();
        let description = rule.info().content(lang).short_description();

        let meaning = match lang {
            NativeLanguage::Russian => format!(
                "Слово: {} с примененной грамматической конструкцией: {}",
                meaning, description
            ),
            NativeLanguage::English => format!(
                "Word: {} with applyed grammar rule: {}",
                meaning, description
            ),
        };

        Ok(Self {
            word: Question::new(formatted_word)?,
            meaning: Answer::new(meaning)?,
            example_phrases: self.example_phrases.clone(),
        })
    }

    pub fn revert(&self) -> Result<Self, OrigaError> {
        Ok(Self {
            word: Question::new(self.meaning.text().to_string())?,
            meaning: Answer::new(self.word.text().to_string())?,
            example_phrases: self.example_phrases.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamplePhrase {
    text: String,
    translation: String,
}

impl ExamplePhrase {
    pub fn new(text: String, translation: String) -> Self {
        Self { text, translation }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn translation(&self) -> &String {
        &self.translation
    }
}
