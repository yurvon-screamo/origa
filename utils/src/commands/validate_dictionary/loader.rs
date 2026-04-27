use std::fs;
use std::path::Path;

use origa::domain::OrigaError;

pub(crate) fn load_vocabulary_chunks(
    base_path: &Path,
) -> Result<Vec<(String, serde_json::Value)>, OrigaError> {
    let vocab_path = base_path.join("cdn").join("dictionary");

    let mut entries = Vec::new();
    let mut chunk_count = 0;

    for entry in fs::read_dir(&vocab_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read directory {}: {}", vocab_path.display(), e),
    })? {
        let entry = entry.map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read entry: {}", e),
        })?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !file_name.starts_with("chunk_") || !file_name.ends_with(".json") {
            continue;
        }
        chunk_count += 1;

        let content = fs::read_to_string(&path).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read {}: {}", path.display(), e),
        })?;

        let json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to parse {}: {}", path.display(), e),
            })?;

        if let Some(obj) = json.as_object() {
            for (word, translations) in obj {
                entries.push((word.clone(), translations.clone()));
            }
        }
    }

    tracing::info!(
        "Loaded {} vocabulary entries from {} chunk files in {}",
        entries.len(),
        chunk_count,
        vocab_path.display()
    );
    Ok(entries)
}
