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

/// Vocabulary database ready for rkyv serialization
/// This is the parsed and processed vocabulary data, ready for fast loading
#[derive(Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct VocabularyDatabaseData {
    pub entries: Vec<(String, String, String)>, // (word, russian_translation, english_translation)
}

impl From<&VocabularyDatabase> for VocabularyDatabaseData {
    fn from(db: &VocabularyDatabase) -> Self {
        Self {
            entries: db
                .vocabulary_map
                .iter()
                .map(|(word, info)| {
                    (
                        word.clone(),
                        info.russian_translation.clone(),
                        info.english_translation.clone(),
                    )
                })
                .collect(),
        }
    }
}

impl From<VocabularyDatabaseData> for VocabularyDatabase {
    fn from(data: VocabularyDatabaseData) -> Self {
        Self {
            vocabulary_map: data
                .entries
                .into_iter()
                .map(|(word, ru, en)| {
                    (
                        word.clone(),
                        VocabularyInfo {
                            word,
                            russian_translation: ru,
                            english_translation: en,
                        },
                    )
                })
                .collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
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
    let start = std::time::Instant::now();
    tracing::info!("📖 Initializing vocabulary dictionary...");
    let db = VocabularyDatabase::from_chunks(data)?;
    tracing::info!(
        "📖 Vocabulary dictionary initialized ({:.2}s)",
        start.elapsed().as_secs_f64()
    );
    VOCABULARY_DICTIONARY
        .set(db)
        .map_err(|_| OrigaError::VocabularyParseError {
            reason: "Failed to set vocabulary dictionary".to_string(),
        })
}

/// Serialize VocabularyDatabase to rkyv bytes
pub fn serialize_vocabulary_to_rkyv(db: &VocabularyDatabase) -> Result<Vec<u8>, OrigaError> {
    let data = VocabularyDatabaseData::from(db);
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&data).map_err(|e| {
        OrigaError::VocabularyParseError {
            reason: format!("Failed to serialize vocabulary: {}", e),
        }
    })?;
    Ok(bytes.to_vec())
}

/// Initialize vocabulary from rkyv bytes
pub fn init_vocabulary_from_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    let start = std::time::Instant::now();
    tracing::info!("📖 Loading vocabulary from rkyv...");

    let archived = rkyv::access::<ArchivedVocabularyDatabaseData, rkyv::rancor::Error>(bytes)
        .map_err(|e| OrigaError::VocabularyParseError {
            reason: format!("Failed to validate vocabulary data: {:?}", e),
        })?;

    tracing::info!(
        "📖 Vocabulary accessed from rkyv ({:.2}s)",
        start.elapsed().as_secs_f64()
    );

    // Convert to owned VocabularyDatabaseData
    let data = VocabularyDatabaseData {
        entries: archived
            .entries
            .iter()
            .map(|e| {
                (
                    e.0.as_str().to_string(),
                    e.1.as_str().to_string(),
                    e.2.as_str().to_string(),
                )
            })
            .collect(),
    };

    let db = VocabularyDatabase::from(data);

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
