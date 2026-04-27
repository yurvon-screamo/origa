use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use origa::domain::OrigaError;

use super::types::{ValidationRecord, ValidationSummary};

pub(crate) fn load_processed_words(output_path: &Path) -> Result<HashSet<String>, OrigaError> {
    if !output_path.exists() {
        return Ok(HashSet::new());
    }

    let content = fs::read_to_string(output_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!(
            "Failed to read progress file {}: {}",
            output_path.display(),
            e
        ),
    })?;

    let mut processed = HashSet::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(record) = serde_json::from_str::<ValidationRecord>(line) {
            processed.insert(record.word);
        }
    }

    tracing::info!(
        "Loaded {} previously processed words for resume",
        processed.len()
    );
    Ok(processed)
}

pub(crate) fn append_record(
    output_path: &Path,
    record: &ValidationRecord,
) -> Result<(), OrigaError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_path)
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to open progress file: {}", e),
        })?;

    let json = serde_json::to_string(record).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to serialize record: {}", e),
    })?;

    writeln!(file, "{}", json).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write record: {}", e),
    })?;

    Ok(())
}

pub(crate) fn generate_summary(output_path: &Path) -> Result<ValidationSummary, OrigaError> {
    let content = fs::read_to_string(output_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!(
            "Failed to read progress file {}: {}",
            output_path.display(),
            e
        ),
    })?;

    let mut total_checked = 0usize;
    let mut total_valid = 0usize;
    let mut total_invalid = 0usize;
    let mut invalid_words = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(record) = serde_json::from_str::<ValidationRecord>(line) {
            total_checked += 1;
            if record.valid {
                total_valid += 1;
            } else {
                total_invalid += 1;
                invalid_words.push(record.word);
            }
        }
    }

    Ok(ValidationSummary {
        total_checked,
        total_valid,
        total_invalid,
        invalid_words,
    })
}
