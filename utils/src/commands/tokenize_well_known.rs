use crate::dictionary::load_dictionary;
use crate::utils::collect_json_files;
use origa::domain::{OrigaError, tokenize_text};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Tokenizes a list of words and returns sorted unique vocabulary words
fn tokenize_words(words: &[String]) -> Result<Vec<String>, OrigaError> {
    let text = words.join(" ");
    let tokens = tokenize_text(&text)?;

    let mut unique_words: HashSet<String> = HashSet::new();
    for token in tokens {
        if token.part_of_speech().is_vocabulary_word() {
            unique_words.insert(token.orthographic_base_form().to_string());
        }
    }

    let mut sorted: Vec<String> = unique_words.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}

/// Extracts words array from JSON map
fn extract_words_array(json: &Map<String, Value>, path: &Path) -> Result<Vec<String>, OrigaError> {
    let words = json
        .get("words")
        .and_then(|v| v.as_array())
        .ok_or_else(|| OrigaError::TokenizerError {
            reason: format!("No 'words' array in {}", path.display()),
        })?;

    Ok(words
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect())
}

/// Processes a single well-known JSON file
fn process_well_known_file(path: &Path) -> Result<(), OrigaError> {
    // Read and parse JSON
    let content = fs::read_to_string(path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {}", path.display(), e),
    })?;

    let mut json: Map<String, Value> =
        serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to parse {}: {}", path.display(), e),
        })?;

    // Extract, tokenize, and update words
    let words_vec = extract_words_array(&json, path)?;
    let new_words = tokenize_words(&words_vec)?;

    json.insert(
        "words".to_string(),
        Value::Array(new_words.into_iter().map(Value::String).collect()),
    );

    // Write back to file
    let output = serde_json::to_string_pretty(&Value::Object(json)).map_err(|e| {
        OrigaError::TokenizerError {
            reason: format!("Failed to serialize {}: {}", path.display(), e),
        }
    })?;

    fs::write(path, output).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write {}: {}", path.display(), e),
    })?;

    tracing::info!("Processed: {}", path.display());
    Ok(())
}

/// Batch processes JSON files in well_known_set format
pub fn run_tokenize_well_known(path: std::path::PathBuf) -> Result<(), OrigaError> {
    load_dictionary()?;

    let files = collect_json_files(&path)?;

    if files.is_empty() {
        eprintln!("JSON файлы не найдены");
        std::process::exit(1);
    }

    tracing::info!("Found {} JSON file(s)", files.len());

    for file in files {
        if let Err(e) = process_well_known_file(&file) {
            eprintln!("Ошибка обработки {}: {}", file.display(), e);
        }
    }

    Ok(())
}
