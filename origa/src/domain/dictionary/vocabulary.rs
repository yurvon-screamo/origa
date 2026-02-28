use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::domain::{OrigaError, value_objects::NativeLanguage};

pub static VOCABULARY_DICTIONARY: OnceLock<VocabularyDatabase> = OnceLock::new();

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

pub struct VocabularyChunkData {
    pub chunk_01: String,
    pub chunk_02: String,
    pub chunk_03: String,
    pub chunk_04: String,
    pub chunk_05: String,
    pub chunk_06: String,
    pub chunk_07: String,
    pub chunk_08: String,
    pub chunk_09: String,
    pub chunk_10: String,
}

pub fn init_vocabulary_dictionary(data: VocabularyChunkData) -> Result<(), OrigaError> {
    let db = VocabularyDatabase::from_chunks(data)?;
    let _ = VOCABULARY_DICTIONARY.set(db);
    Ok(())
}

pub fn is_vocabulary_loaded() -> bool {
    VOCABULARY_DICTIONARY.get().is_some()
}

pub fn get_translation(word: &str, native_language: &NativeLanguage) -> Option<String> {
    VOCABULARY_DICTIONARY
        .get()
        .and_then(|db| db.get_translation(word, native_language))
}

impl VocabularyDatabase {
    fn from_chunks(data: VocabularyChunkData) -> Result<Self, OrigaError> {
        let parse_chunk = |json: &str, chunk_name: &str| {
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(json).map_err(|e| {
                OrigaError::VocabularyParseError {
                    reason: format!("Failed to parse {}: {}", chunk_name, e),
                }
            })
        };

        let vocabulary_data: HashMap<_, _> = parse_chunk(&data.chunk_01, "chunk_01")?
            .into_iter()
            .chain(parse_chunk(&data.chunk_02, "chunk_02")?)
            .chain(parse_chunk(&data.chunk_03, "chunk_03")?)
            .chain(parse_chunk(&data.chunk_04, "chunk_04")?)
            .chain(parse_chunk(&data.chunk_05, "chunk_05")?)
            .chain(parse_chunk(&data.chunk_06, "chunk_06")?)
            .chain(parse_chunk(&data.chunk_07, "chunk_07")?)
            .chain(parse_chunk(&data.chunk_08, "chunk_08")?)
            .chain(parse_chunk(&data.chunk_09, "chunk_09")?)
            .chain(parse_chunk(&data.chunk_10, "chunk_10")?)
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

        Ok(Self { vocabulary_map })
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
