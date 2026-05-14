use origa::domain::OrigaError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct KanjiEntry {
    pub kanji: String,
    #[serde(default)]
    pub on_readings: Vec<String>,
    #[serde(default)]
    pub kun_readings: Vec<String>,
    #[serde(flatten)]
    pub other: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
pub struct KanjiDictionary {
    pub kanji: Vec<KanjiEntry>,
}

pub fn read_dictionary(path: &PathBuf) -> Result<KanjiDictionary, OrigaError> {
    let content = fs::read_to_string(path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {}", path.display(), e),
    })?;
    serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to parse kanji dictionary: {}", e),
    })
}

pub fn create_backup(path: &PathBuf) -> Result<PathBuf, OrigaError> {
    let backup_path = path.with_extension("json.bak");
    fs::copy(path, &backup_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to create backup {}: {}", backup_path.display(), e),
    })?;
    Ok(backup_path)
}

pub fn write_dictionary(path: &PathBuf, dictionary: &KanjiDictionary) -> Result<(), OrigaError> {
    let json =
        serde_json::to_string_pretty(dictionary).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to serialize kanji dictionary: {}", e),
        })?;
    fs::write(path, json).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write {}: {}", path.display(), e),
    })
}
