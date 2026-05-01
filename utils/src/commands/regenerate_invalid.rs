use std::collections::HashSet;
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

fn rewrite_jsonl_without_words(
    jsonl_path: &Path,
    remaining_words: &HashSet<&str>,
) -> Result<(), OrigaError> {
    let content = fs::read_to_string(jsonl_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {}", jsonl_path.display(), e),
    })?;

    let mut kept_lines = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(record) = serde_json::from_str::<ValidationRecord>(line) {
            if remaining_words.contains(record.word.as_str()) {
                kept_lines.push(line.to_string());
            }
        }
    }

    let output = kept_lines.join("\n");
    fs::write(jsonl_path, output).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write {}: {}", jsonl_path.display(), e),
    })?;

    Ok(())
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

fn save_modified_chunks(
    chunks: &[ChunkFile],
    modified_indices: &HashSet<usize>,
) -> Result<(), OrigaError> {
    for &idx in modified_indices {
        save_chunk(&chunks[idx])?;
    }
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

    let mut invalid_words = load_invalid_words(&input)?;
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

    let total = invalid_words.len();
    let batch_size = workers * 20;
    let mut global_processed = 0usize;
    let mut global_updated = 0usize;
    let mut global_failed = 0usize;

    while !invalid_words.is_empty() {
        let batch_end = std::cmp::min(batch_size, invalid_words.len());
        let batch: Vec<String> = invalid_words.drain(..batch_end).collect();
        let batch_total = batch.len();

        tracing::info!(
            "Processing batch of {} words ({}/{} remaining)...",
            batch_total,
            global_processed + batch_total,
            total
        );

        let semaphore = Arc::new(Semaphore::new(workers));
        let mut handles = Vec::new();

        for word in batch {
            let semaphore = Arc::clone(&semaphore);
            let api_base = api_base.clone();
            let api_key = api_key.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.expect("semaphore closed");

                let entry =
                    translate_word(&word, &api_base, &api_key, to_russian, to_english).await;

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

        let mut modified_indices = HashSet::new();
        let mut successfully_processed = Vec::new();
        let mut failed_words = Vec::new();

        let mut batch_updated = 0usize;
        let mut batch_failed = 0usize;

        for (word, result) in results {
            match result {
                Ok(entry) => {
                    if entry.russian_translation.is_none() && entry.english_translation.is_none() {
                        tracing::warn!("Empty translation for '{}', keeping in queue", word);
                        batch_failed += 1;
                        failed_words.push(word);
                        continue;
                    }

                    let mut found = false;
                    for (idx, chunk) in chunks.iter_mut().enumerate() {
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
                                modified_indices.insert(idx);
                                found = true;
                                batch_updated += 1;
                                successfully_processed.push(word.clone());
                                tracing::info!("Updated: {}", word);
                                break;
                            }
                        }
                    }

                    if !found {
                        tracing::warn!("Word '{}' not found in any chunk, keeping in queue", word);
                        batch_failed += 1;
                        failed_words.push(word);
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to translate '{}': {}, keeping in queue", word, e);
                    batch_failed += 1;
                    failed_words.push(word);
                },
            }
        }

        // Save modified chunks to disk
        save_modified_chunks(&chunks, &modified_indices)?;
        tracing::info!("Saved {} modified chunk files", modified_indices.len());

        // Put failed words back into the queue (at the front, so they're retried first)
        if !failed_words.is_empty() {
            let rest = invalid_words.split_off(0);
            invalid_words = failed_words;
            invalid_words.extend(rest);
        }

        // Rewrite the input JSONL: keep only words still in the queue
        let remaining_words: HashSet<&str> = invalid_words.iter().map(|s| s.as_str()).collect();
        rewrite_jsonl_without_words(&input, &remaining_words)?;
        tracing::info!(
            "Updated input file: {} words removed, {} remaining",
            successfully_processed.len(),
            invalid_words.len()
        );

        global_processed += batch_total;
        global_updated += batch_updated;
        global_failed += batch_failed;

        tracing::info!(
            "Batch complete: {} updated, {} failed. Total progress: {}/{} ({:.1}%)",
            batch_updated,
            batch_failed,
            global_processed,
            total,
            global_processed as f64 / total as f64 * 100.0
        );
    }

    tracing::info!("=== Regeneration Complete ===");
    tracing::info!("Updated: {} words", global_updated);
    tracing::info!("Failed: {} words", global_failed);
    tracing::info!("Input file is now empty (all words processed)");

    Ok(())
}
