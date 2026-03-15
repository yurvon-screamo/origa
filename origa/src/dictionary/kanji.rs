use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError};

pub static KANJI_DICTIONARY: OnceLock<KanjiDatabase> = OnceLock::new();

#[derive(Clone, Serialize, Deserialize)]
pub struct KanjiData {
    pub kanji_json: String,
}

pub fn init_kanji(data: KanjiData) -> Result<(), OrigaError> {
    let db = KanjiDatabase::from_json(&data.kanji_json)?;
    KANJI_DICTIONARY
        .set(db)
        .map_err(|_| OrigaError::KradfileError {
            reason: "Failed to set kanji dictionary".to_string(),
        })
}

pub fn is_kanji_loaded() -> bool {
    KANJI_DICTIONARY.get().is_some()
}

pub fn get_kanji_info(kanji: &str) -> Result<&'static KanjiInfo, OrigaError> {
    KANJI_DICTIONARY
        .get()
        .ok_or(OrigaError::KradfileError {
            reason: "Kanji dictionary not loaded".to_string(),
        })?
        .get_kanji_info(kanji)
}

pub fn get_kanji_list(level: &JapaneseLevel) -> Vec<&'static KanjiInfo> {
    KANJI_DICTIONARY
        .get()
        .map(|db| db.get_kanji_list(level))
        .unwrap_or_default()
}

pub struct KanjiDatabase {
    kanji_map: HashMap<String, KanjiInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopularWord {
    word: String,
    translation: String,
}

impl PopularWord {
    pub fn new(word: String, translation: String) -> Self {
        Self { word, translation }
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn translation(&self) -> &str {
        &self.translation
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanjiInfo {
    kanji: char,
    jlpt: JapaneseLevel,
    used_in: u32,
    description: String,
    radicals: Vec<char>,
    popular_words: Vec<String>,
    on_readings: Vec<String>,
    kun_readings: Vec<String>,
}

impl KanjiInfo {
    pub fn kanji(&self) -> char {
        self.kanji
    }

    pub fn jlpt(&self) -> &JapaneseLevel {
        &self.jlpt
    }

    pub fn used_in(&self) -> u32 {
        self.used_in
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn radicals_chars(&self) -> &[char] {
        &self.radicals
    }

    pub fn popular_words(&self) -> &[String] {
        &self.popular_words
    }

    pub fn on_readings(&self) -> &[String] {
        &self.on_readings
    }

    pub fn kun_readings(&self) -> &[String] {
        &self.kun_readings
    }

    pub fn popular_words_with_translations(
        &self,
        native_language: &NativeLanguage,
    ) -> Vec<PopularWord> {
        use crate::dictionary::vocabulary::VOCABULARY_DICTIONARY;

        self.popular_words
            .iter()
            .map(|word| {
                let translation = VOCABULARY_DICTIONARY
                    .get()
                    .and_then(|db| db.get_translation(word, native_language))
                    .unwrap_or_else(|| "Перевод не найден".to_string());
                PopularWord::new(word.clone(), translation)
            })
            .collect()
    }
}

impl KanjiDatabase {
    fn from_json(json: &str) -> Result<Self, OrigaError> {
        let kanji_db: KanjiDatabaseStoredType =
            serde_json::from_str(json).map_err(|e| OrigaError::KradfileError {
                reason: format!("Failed to parse kanji.json: {}", e),
            })?;

        let kanji_map = kanji_db
            .kanji
            .into_iter()
            .map(|k| {
                let jlpt = JapaneseLevel::from_str_or_default(&k.jlpt);
                let kanji_char = k.kanji.chars().next().unwrap();
                let radicals = k
                    .radicals
                    .into_iter()
                    .flat_map(|r| r.chars().collect::<Vec<_>>())
                    .collect::<Vec<char>>();

                (
                    kanji_char.to_string(),
                    KanjiInfo {
                        kanji: kanji_char,
                        jlpt,
                        used_in: k.used_in,
                        description: k.description,
                        radicals,
                        popular_words: k.popular_words,
                        on_readings: k.on_readings,
                        kun_readings: k.kun_readings,
                    },
                )
            })
            .collect::<HashMap<String, KanjiInfo>>();

        Ok(Self { kanji_map })
    }

    pub fn get_kanji_info(&self, kanji: &str) -> Result<&KanjiInfo, OrigaError> {
        self.kanji_map.get(kanji).ok_or(OrigaError::KradfileError {
            reason: format!("Kanji {} not found in kanji database", kanji),
        })
    }

    pub fn get_kanji_list(&self, level: &JapaneseLevel) -> Vec<&KanjiInfo> {
        self.kanji_map
            .values()
            .filter(|x| x.jlpt() == level)
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
struct KanjiStoredType {
    kanji: String,
    jlpt: String,
    used_in: u32,
    description: String,
    radicals: Vec<String>,
    popular_words: Vec<String>,
    #[serde(default)]
    on_readings: Vec<String>,
    #[serde(default)]
    kun_readings: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct KanjiDatabaseStoredType {
    kanji: Vec<KanjiStoredType>,
}
