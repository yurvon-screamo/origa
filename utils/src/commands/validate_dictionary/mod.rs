mod loader;
mod progress;
mod types;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use origa::domain::OrigaError;

use crate::api::validate_translation;
use crate::utils::get_base_path;

use loader::load_vocabulary_chunks;
use progress::{append_record, generate_summary, load_processed_words};
use types::ValidationRecord;

fn extract_translations(translations: &serde_json::Value) -> (String, String) {
    let ru = translations
        .get("russian_translation")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let en = translations
        .get("english_translation")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    (ru, en)
}

#[allow(clippy::too_many_arguments)]
async fn validate_word(
    word: String,
    ru: String,
    en: String,
    api_base: String,
    api_key: String,
    model: String,
    output_path: Arc<PathBuf>,
    file_lock: Arc<StdMutex<()>>,
) -> bool {
    let (valid, raw_response) = validate_translation(&api_base, &api_key, &model, &word, &ru, &en)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to validate '{}': {}", word, e);
            (true, format!("Error: {}", e))
        });

    let record = ValidationRecord {
        word: word.clone(),
        valid,
        raw_response: Some(raw_response),
    };

    let _guard = file_lock.lock().expect("file lock poisoned");
    if let Err(e) = append_record(&output_path, &record) {
        tracing::error!("Failed to save record for '{}': {}", word, e);
    }

    valid
}

#[allow(clippy::too_many_arguments)]
pub async fn run_validate_dictionary(
    api_key: String,
    api_base: String,
    model: String,
    workers: usize,
    output: Option<PathBuf>,
    dry_run: bool,
    limit: Option<usize>,
) -> Result<(), OrigaError> {
    let base_path = get_base_path();

    let output_path = output.unwrap_or_else(|| base_path.join("invalid_vocabulary.jsonl"));
    let mut entries = load_vocabulary_chunks(&base_path)?;

    let processed = load_processed_words(&output_path)?;
    if !processed.is_empty() {
        tracing::info!(
            "Resuming: {} words already validated in previous run",
            processed.len()
        );
    }

    entries.retain(|(word, _)| !processed.contains(word));

    if let Some(limit) = limit {
        entries.truncate(limit);
    }

    tracing::info!(
        "Words to validate: {} (skipped {} already processed)",
        entries.len(),
        processed.len()
    );

    if dry_run {
        tracing::info!("Dry run — no API calls will be made");
        tracing::info!("Estimated API calls: {} (model: {})", entries.len(), model);
        tracing::info!("Output would be saved to: {}", output_path.display());
        return Ok(());
    }

    let semaphore = Arc::new(tokio::sync::Semaphore::new(workers));
    let checked = Arc::new(AtomicUsize::new(0));
    let total = entries.len();
    let output_path = Arc::new(output_path);
    let file_lock = Arc::new(StdMutex::new(()));
    let mut handles = Vec::new();

    for (word, translations) in entries {
        let semaphore = Arc::clone(&semaphore);
        let checked = Arc::clone(&checked);
        let output_path = Arc::clone(&output_path);
        let file_lock = Arc::clone(&file_lock);
        let (ru, en) = extract_translations(&translations);

        let api_key = api_key.clone();
        let api_base = api_base.clone();
        let model = model.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.expect("semaphore closed");

            let valid = validate_word(
                word.clone(),
                ru,
                en,
                api_base,
                api_key,
                model,
                output_path,
                file_lock,
            )
            .await;

            let checked_count = checked.fetch_add(1, Ordering::Relaxed) + 1;
            if checked_count % 100 == 0 || checked_count == total {
                tracing::info!(
                    "Progress: {}/{} words validated ({:.1}%)",
                    checked_count,
                    total,
                    checked_count as f64 / total as f64 * 100.0
                );
            }

            valid
        });

        handles.push(handle);
    }

    for result in futures::future::join_all(handles).await {
        if let Err(e) = result {
            tracing::error!("Task join error: {}", e);
        }
    }

    let summary = generate_summary(&output_path)?;

    tracing::info!("=== Validation Complete ===");
    tracing::info!("Total checked: {}", summary.total_checked);
    tracing::info!("Valid: {}", summary.total_valid);
    tracing::info!("Invalid: {}", summary.total_invalid);

    if !summary.invalid_words.is_empty() {
        tracing::info!("Invalid words saved to: {}", output_path.display());

        let list_path = output_path.with_extension("json");
        let list_json =
            serde_json::to_string_pretty(&summary).map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to serialize summary: {}", e),
            })?;
        fs::write(&list_path, list_json).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to write summary file: {}", e),
        })?;
        tracing::info!("Summary saved to: {}", list_path.display());
    }

    Ok(())
}
