use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use origa::domain::OrigaError;
use tokio::sync::Semaphore;

use crate::api::translate_word;
use crate::utils::get_base_path;

#[derive(Debug, serde::Deserialize)]
struct ValidationRecord {
    word: String,
    valid: bool,
}

fn load_invalid_words(jsonl_path: &Path) -> Result<Vec<String>, OrigaError> {
    if !jsonl_path.exists() {
        return Err(OrigaError::TokenizerError {
            reason: format!("Progress file not found: {}", jsonl_path.display()),
        });
    }

    let content = fs::read_to_string(jsonl_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {}", jsonl_path.display(), e),
    })?;

    let mut invalid_words = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(record) = serde_json::from_str::<ValidationRecord>(line) {
            if !record.valid {
                invalid_words.push(record.word);
            }
        }
    }

    Ok(invalid_words)
}

struct ChunkFile {
    path: PathBuf,
    data: serde_json::Map<String, serde_json::Value>,
}

fn load_all_chunks(vocab_path: &Path) -> Result<Vec<ChunkFile>, OrigaError> {
    let mut chunks = Vec::new();

    for entry in fs::read_dir(vocab_path).map_err(|e| OrigaError::TokenizerError {
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

        let content = fs::read_to_string(&path).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read {}: {}", path.display(), e),
        })?;

        let json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to parse {}: {}", path.display(), e),
            })?;

        if let Some(obj) = json.as_object() {
            chunks.push(ChunkFile {
                path,
                data: obj.clone(),
            });
        }
    }

    Ok(chunks)
}

fn save_chunk(chunk: &ChunkFile) -> Result<(), OrigaError> {
    let json =
        serde_json::to_string_pretty(&chunk.data).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to serialize chunk {}: {}", chunk.path.display(), e),
        })?;

    fs::write(&chunk.path, json).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write {}: {}", chunk.path.display(), e),
    })?;

    Ok(())
}

pub async fn run_regenerate_invalid(
    input: PathBuf,
    api_base: String,
    api_key: String,
    workers: usize,
    dry_run: bool,
    russian_only: bool,
    english_only: bool,
) -> Result<(), OrigaError> {
    let base_path = get_base_path();
    let vocab_path = base_path.join("cdn").join("dictionary");

    let invalid_words = load_invalid_words(&input)?;
    if invalid_words.is_empty() {
        tracing::info!("No invalid words found in {}", input.display());
        return Ok(());
    }

    tracing::info!("Found {} invalid words to regenerate", invalid_words.len());

    if dry_run {
        tracing::info!("Dry run — no changes will be made");
        for word in &invalid_words {
            tracing::info!("  Would regenerate: {}", word);
        }
        return Ok(());
    }

    let mut chunks = load_all_chunks(&vocab_path)?;
    tracing::info!("Loaded {} chunk files", chunks.len());

    let to_russian = !english_only;
    let to_english = !russian_only;

    let semaphore = Arc::new(Semaphore::new(workers));
    let mut handles = Vec::new();
    let total = invalid_words.len();
    let processed = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    for word in invalid_words {
        let semaphore = Arc::clone(&semaphore);
        let processed = Arc::clone(&processed);
        let api_base = api_base.clone();
        let api_key = api_key.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.expect("semaphore closed");

            let entry = translate_word(&word, &api_base, &api_key, to_russian, to_english).await;

            let count = processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            if count % 50 == 0 || count == total {
                tracing::info!(
                    "Progress: {}/{} ({:.1}%)",
                    count,
                    total,
                    count as f64 / total as f64 * 100.0
                );
            }

            (word, entry)
        });

        handles.push(handle);
    }

    let results: Vec<(String, Result<crate::api::VocabularyEntry, OrigaError>)> =
        futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.expect("task panicked"))
            .collect();

    let mut updated_count = 0;
    let mut failed_count = 0;

    for (word, result) in results {
        match result {
            Ok(entry) => {
                if entry.russian_translation.is_none() && entry.english_translation.is_none() {
                    tracing::warn!("Empty translation for '{}', skipping", word);
                    failed_count += 1;
                    continue;
                }

                let mut found = false;
                for chunk in &mut chunks {
                    if let Some(translations) = chunk.data.get_mut(&word) {
                        if let Some(obj) = translations.as_object_mut() {
                            if let Some(ru) = &entry.russian_translation {
                                obj.insert(
                                    "russian_translation".to_string(),
                                    serde_json::json!(ru),
                                );
                            }
                            if let Some(en) = &entry.english_translation {
                                obj.insert(
                                    "english_translation".to_string(),
                                    serde_json::json!(en),
                                );
                            }
                            found = true;
                            updated_count += 1;
                            tracing::info!("Updated: {}", word);
                            break;
                        }
                    }
                }

                if !found {
                    tracing::warn!("Word '{}' not found in any chunk", word);
                    failed_count += 1;
                }
            },
            Err(e) => {
                tracing::error!("Failed to translate '{}': {}", word, e);
                failed_count += 1;
            },
        }
    }

    let mut saved_chunks = 0;
    for chunk in &chunks {
        save_chunk(chunk)?;
        saved_chunks += 1;
    }

    tracing::info!("=== Regeneration Complete ===");
    tracing::info!("Updated: {} words", updated_count);
    tracing::info!("Failed: {} words", failed_count);
    tracing::info!("Saved: {} chunk files", saved_chunks);

    Ok(())
}
