use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::domain::value_objects::NativeLanguage;

pub static VOCABULARY_DICTIONARY: LazyLock<VocabularyDatabase> =
    LazyLock::new(VocabularyDatabase::new);

#[derive(Debug, Clone)]
pub struct VocabularyInfo {
    word: String,
    russian_translation: String,
    english_translation: String,
}

impl VocabularyInfo {
    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn russian_translation(&self) -> &str {
        &self.russian_translation
    }

    pub fn english_translation(&self) -> &str {
        &self.english_translation
    }
}

#[derive(Serialize, Deserialize)]
struct VocabularyEntryStoredType {
    #[serde(default)]
    level: Option<String>,
    russian_translation: String,
    english_translation: String,
}

pub struct VocabularyDatabase {
    vocabulary_map: HashMap<String, VocabularyInfo>,
}

impl Default for VocabularyDatabase {
    fn default() -> Self {
        Self::new()
    }
}

static CHUNK_01_STR: &str = include_str!("vocabulary/chunk_01.json");
static CHUNK_02_STR: &str = include_str!("vocabulary/chunk_02.json");
static CHUNK_03_STR: &str = include_str!("vocabulary/chunk_03.json");
static CHUNK_04_STR: &str = include_str!("vocabulary/chunk_04.json");
static CHUNK_05_STR: &str = include_str!("vocabulary/chunk_05.json");
static CHUNK_06_STR: &str = include_str!("vocabulary/chunk_06.json");
static CHUNK_07_STR: &str = include_str!("vocabulary/chunk_07.json");
static CHUNK_08_STR: &str = include_str!("vocabulary/chunk_08.json");
static CHUNK_09_STR: &str = include_str!("vocabulary/chunk_09.json");
static CHUNK_10_STR: &str = include_str!("vocabulary/chunk_10.json");

impl VocabularyDatabase {
    pub fn new() -> Self {
        let vocabulary_data: HashMap<_, _> = serde_json::from_str::<
            HashMap<String, VocabularyEntryStoredType>,
        >(CHUNK_01_STR)
        .unwrap()
        .into_iter()
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(CHUNK_02_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(CHUNK_03_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(CHUNK_04_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(CHUNK_05_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(CHUNK_06_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(CHUNK_07_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(CHUNK_08_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(CHUNK_09_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(CHUNK_10_STR)
                .unwrap(),
        )
        .collect();

        let vocabulary_map = vocabulary_data
            .into_iter()
            .map(|(word, entry)| {
                (
                    word.clone(),
                    VocabularyInfo {
                        word,
                        russian_translation: entry.russian_translation,
                        english_translation: entry.english_translation,
                    },
                )
            })
            .collect::<HashMap<String, VocabularyInfo>>();

        Self { vocabulary_map }
    }

    pub fn get_translation(&self, word: &str, native_language: &NativeLanguage) -> Option<String> {
        self.vocabulary_map
            .get(word)
            .map(|info| match native_language {
                NativeLanguage::Russian => info.russian_translation.clone(),
                NativeLanguage::English => info.english_translation.clone(),
            })
    }

    pub fn get_vocabulary_info(&self, word: &str) -> Option<&VocabularyInfo> {
        self.vocabulary_map.get(word)
    }
}
