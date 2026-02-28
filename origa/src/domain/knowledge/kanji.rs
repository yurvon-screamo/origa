use crate::domain::{
    dictionary::{get_kanji_info, get_radical_info, get_translation, RadicalInfo},
    value_objects::{JapaneseLevel, NativeLanguage},
    Answer, OrigaError, Question,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanjiCard {
    kanji: Question,
    description: Answer,
    example_words: Vec<ExampleKanjiWord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExampleKanjiWord {
    word: String,
    meaning: String,
}

impl KanjiCard {
    pub fn new(kanji: String, native_language: &NativeLanguage) -> Result<Self, OrigaError> {
        let kanji_info = get_kanji_info(&kanji)?;
        let description = kanji_info.description();
        let example_words = kanji_info
            .popular_words()
            .iter()
            .map(|word| {
                let meaning = get_translation(word, native_language).unwrap_or_default();

                ExampleKanjiWord {
                    word: word.clone(),
                    meaning,
                }
            })
            .collect();

        Ok(Self {
            kanji: Question::new(kanji.to_string())?,
            description: Answer::new(description.to_string())?,
            example_words,
        })
    }

    pub fn kanji(&self) -> &Question {
        &self.kanji
    }

    pub fn description(&self) -> &Answer {
        &self.description
    }

    pub fn example_words(&self) -> &[ExampleKanjiWord] {
        &self.example_words
    }

    pub fn jlpt(&self) -> JapaneseLevel {
        get_kanji_info(self.kanji.text())
            .map(|kanji_info| kanji_info.jlpt().to_owned())
            .unwrap_or(JapaneseLevel::N1)
    }

    pub fn used_in(&self) -> u32 {
        get_kanji_info(self.kanji.text())
            .map(|kanji_info| kanji_info.used_in())
            .unwrap_or(0)
    }

    pub fn radicals_info(&self) -> Result<Vec<&'static RadicalInfo>, OrigaError> {
        let kanji_info = get_kanji_info(self.kanji.text())?;
        kanji_info
            .radicals_chars()
            .iter()
            .map(|&r| get_radical_info(r))
            .collect()
    }
}

impl ExampleKanjiWord {
    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn meaning(&self) -> &str {
        &self.meaning
    }
}
