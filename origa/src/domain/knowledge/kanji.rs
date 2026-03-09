use crate::domain::{
    Answer, FALLBACK_ANSWER, OrigaError, Question,
    dictionary::{RadicalInfo, get_kanji_info, get_radical_info, get_translation},
    value_objects::{JapaneseLevel, NativeLanguage},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanjiCard {
    kanji: Question,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExampleKanjiWord {
    word: String,
    meaning: String,
}

impl KanjiCard {
    pub fn new(kanji: String) -> Result<Self, OrigaError> {
        get_kanji_info(&kanji)?;
        Ok(Self {
            kanji: Question::new(kanji)?,
        })
    }

    pub fn kanji(&self) -> &Question {
        &self.kanji
    }

    pub fn description(&self) -> Answer {
        get_kanji_info(self.kanji.text())
            .map(|info| {
                Answer::new(info.description().to_string())
                    .unwrap_or_else(|_| Answer::new(FALLBACK_ANSWER.to_string()).unwrap())
            })
            .unwrap_or_else(|_| Answer::new(FALLBACK_ANSWER.to_string()).unwrap())
    }

    pub fn example_words(&self, lang: &NativeLanguage) -> Vec<ExampleKanjiWord> {
        get_kanji_info(self.kanji.text())
            .map(|info| {
                info.popular_words()
                    .iter()
                    .filter_map(|word| {
                        let meaning = get_translation(word, lang).unwrap_or_default();
                        if meaning.is_empty() {
                            None
                        } else {
                            Some(ExampleKanjiWord {
                                word: word.clone(),
                                meaning,
                            })
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
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

    pub fn on_readings(&self) -> Vec<String> {
        get_kanji_info(self.kanji.text())
            .map(|info| info.on_readings().to_vec())
            .unwrap_or_default()
    }

    pub fn kun_readings(&self) -> Vec<String> {
        get_kanji_info(self.kanji.text())
            .map(|info| info.kun_readings().to_vec())
            .unwrap_or_default()
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

impl KanjiCard {
    #[cfg(test)]
    pub fn new_test(kanji: String) -> Self {
        Self {
            kanji: Question::new(kanji).unwrap(),
        }
    }
}
