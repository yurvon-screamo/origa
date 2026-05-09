use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::domain::{NativeLanguage, OrigaError};

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

#[derive(Clone, Serialize, Deserialize)]
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
    pub chunk_11: String,
}

pub fn init_vocabulary(data: VocabularyChunkData) -> Result<(), OrigaError> {
    let db = VocabularyDatabase::from_chunks(data)?;
    VOCABULARY_DICTIONARY
        .set(db)
        .map_err(|_| OrigaError::VocabularyParseError {
            reason: "Failed to set vocabulary dictionary".to_string(),
        })
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
        fn strip_bom(json: &str) -> &str {
            json.strip_prefix('\u{FEFF}').unwrap_or(json)
        }

        let parse_chunk = |json: &str, chunk_name: &str| {
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(strip_bom(json))
                .map_err(|e| OrigaError::VocabularyParseError {
                    reason: format!("Failed to parse {}: {}", chunk_name, e),
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
            .chain(parse_chunk(&data.chunk_11, "chunk_11")?)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::NativeLanguage;

    fn empty_chunk_data_with(chunk_01: &str) -> VocabularyChunkData {
        let empty = "{}".to_string();
        VocabularyChunkData {
            chunk_01: chunk_01.to_string(),
            chunk_02: empty.clone(),
            chunk_03: empty.clone(),
            chunk_04: empty.clone(),
            chunk_05: empty.clone(),
            chunk_06: empty.clone(),
            chunk_07: empty.clone(),
            chunk_08: empty.clone(),
            chunk_09: empty.clone(),
            chunk_10: empty.clone(),
            chunk_11: empty,
        }
    }

    fn make_valid_chunk_json() -> String {
        r#"{
            "猫": {
                "level": "N5",
                "russian_translation": "кошка",
                "english_translation": "cat"
            },
            "犬": {
                "level": "N5",
                "russian_translation": "собака",
                "english_translation": "dog"
            }
        }"#
        .to_string()
    }

    #[test]
    fn from_chunks_valid_json_loads_entries() {
        // Arrange
        let data = empty_chunk_data_with(&make_valid_chunk_json());

        // Act
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        // Assert
        assert!(db.get_vocabulary_info("猫").is_some());
        assert!(db.get_vocabulary_info("犬").is_some());
        assert!(db.get_vocabulary_info("魚").is_none());
    }

    #[test]
    fn from_chunks_strips_bom_prefix() {
        // Arrange
        let json_with_bom = format!("\u{FEFF}{}", make_valid_chunk_json());
        let data = empty_chunk_data_with(&json_with_bom);

        // Act
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        // Assert
        assert!(db.get_vocabulary_info("猫").is_some());
    }

    #[test]
    fn from_chunks_empty_all_chunks_succeeds() {
        // Arrange
        let empty = "{}".to_string();
        let data = VocabularyChunkData {
            chunk_01: empty.clone(),
            chunk_02: empty.clone(),
            chunk_03: empty.clone(),
            chunk_04: empty.clone(),
            chunk_05: empty.clone(),
            chunk_06: empty.clone(),
            chunk_07: empty.clone(),
            chunk_08: empty.clone(),
            chunk_09: empty.clone(),
            chunk_10: empty.clone(),
            chunk_11: empty,
        };

        // Act
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        // Assert
        assert!(db.get_vocabulary_info("anything").is_none());
    }

    #[test]
    fn get_translation_found_returns_correct_language() {
        // Arrange
        let data = empty_chunk_data_with(&make_valid_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        // Act
        let ru = db.get_translation("猫", &NativeLanguage::Russian);
        let en = db.get_translation("猫", &NativeLanguage::English);

        // Assert
        assert_eq!(ru.as_deref(), Some("кошка"));
        assert_eq!(en.as_deref(), Some("cat"));
    }

    #[test]
    fn get_translation_not_found_returns_none() {
        // Arrange
        let data = empty_chunk_data_with(&make_valid_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        // Act
        let result = db.get_translation("魚", &NativeLanguage::Russian);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn from_chunks_invalid_json_returns_error() {
        // Arrange
        let data = empty_chunk_data_with("not valid json");

        // Act
        let result = VocabularyDatabase::from_chunks(data);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(OrigaError::VocabularyParseError { .. })
        ));
    }
}
