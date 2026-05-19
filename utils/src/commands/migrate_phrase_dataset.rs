use origa::domain::OrigaError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use ulid::Ulid;

#[derive(Deserialize)]
struct OldPhraseEntry {
    id: u64,
    text: String,
    audio_file: String,
    tokens: Vec<String>,
}

#[derive(Deserialize)]
struct OldDataset {
    phrases: Vec<OldPhraseEntry>,
}

#[derive(Serialize)]
struct NewPhraseEntry {
    id: String,
    text: String,
    audio_file: String,
    tokens: Vec<String>,
}

#[derive(Serialize)]
struct NewDataset {
    phrases: Vec<NewPhraseEntry>,
}

fn extract_audio_filename(audio_file: &str) -> String {
    PathBuf::from(audio_file)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| audio_file.to_string())
}

pub fn run_migrate_phrase_dataset(dataset_path: PathBuf) -> Result<(), OrigaError> {
    let audio_dir = dataset_path.parent().unwrap_or(&dataset_path).join("audio");

    tracing::info!("Reading dataset: {}", dataset_path.display());
    let content = fs::read_to_string(&dataset_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {}", dataset_path.display(), e),
    })?;

    let old_dataset: OldDataset =
        serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to parse dataset: {}", e),
        })?;

    let total = old_dataset.phrases.len();
    tracing::info!("Loaded {} phrases", total);

    let mut new_phrases: Vec<NewPhraseEntry> = Vec::with_capacity(total);
    let mut ulid_set: HashSet<String> = HashSet::with_capacity(total);

    for phrase in &old_dataset.phrases {
        let ulid = Ulid::new();
        let ulid_str = ulid.to_string();

        if !ulid_set.insert(ulid_str.clone()) {
            return Err(OrigaError::TokenizerError {
                reason: format!("ULID collision: {}", ulid_str),
            });
        }

        let old_filename = extract_audio_filename(&phrase.audio_file);
        let old_audio_path = audio_dir.join(&old_filename);
        let new_filename = format!("{}.mp3", ulid_str);
        let new_audio_path = audio_dir.join(&new_filename);

        if old_audio_path.exists() {
            fs::rename(&old_audio_path, &new_audio_path).map_err(|e| {
                OrigaError::TokenizerError {
                    reason: format!(
                        "Failed to rename {} → {}: {}",
                        old_audio_path.display(),
                        new_audio_path.display(),
                        e
                    ),
                }
            })?;
        } else {
            tracing::warn!(
                "Audio file not found, skipping rename: {}",
                old_audio_path.display()
            );
        }

        new_phrases.push(NewPhraseEntry {
            id: ulid_str,
            text: phrase.text.clone(),
            audio_file: new_filename,
            tokens: phrase.tokens.clone(),
        });
    }

    let new_dataset = NewDataset {
        phrases: new_phrases,
    };

    let backup_path = dataset_path.with_extension("json.bak");
    fs::copy(&dataset_path, &backup_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to create backup {}: {}", backup_path.display(), e),
    })?;
    tracing::info!("Backup saved: {}", backup_path.display());

    let json =
        serde_json::to_string_pretty(&new_dataset).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to serialize dataset: {}", e),
        })?;
    fs::write(&dataset_path, json).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write {}: {}", dataset_path.display(), e),
    })?;

    tracing::info!("Migration complete!");
    tracing::info!("  Total phrases: {}", total);
    tracing::info!("  Unique ULIDs: {}", ulid_set.len());
    if let Some(first) = old_dataset.phrases.first() {
        tracing::info!("  First old id: {}", first.id);
    }
    if let Some(new_first) = new_dataset.phrases.first() {
        tracing::info!("  First new ULID: {}", new_first.id);
    }
    if let Some(new_last) = new_dataset.phrases.last() {
        tracing::info!("  Last new ULID: {}", new_last.id);
    }

    Ok(())
}
