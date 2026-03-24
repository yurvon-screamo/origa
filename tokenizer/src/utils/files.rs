use origa::domain::OrigaError;
use std::fs;
use std::path::{Path, PathBuf};

/// Recursively collects all JSON files from a given path
pub fn collect_json_files(path: &Path) -> Result<Vec<PathBuf>, OrigaError> {
    let mut files = Vec::new();
    collect_json_files_recursive(path, &mut files)?;
    Ok(files)
}

/// Recursive helper function to collect JSON files
fn collect_json_files_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), OrigaError> {
    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path.to_path_buf());
        }
    } else if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read dir {}: {}", path.display(), e),
        })? {
            let entry = entry.map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to read entry: {}", e),
            })?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                collect_json_files_recursive(&entry_path, files)?;
            } else if entry_path.extension().is_some_and(|ext| ext == "json") {
                files.push(entry_path);
            }
        }
    }

    Ok(())
}

/// Gets the base project path (parent of tokenizer directory)
pub fn get_base_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(manifest_dir);

    if path.ends_with("tokenizer") {
        path.pop();
    }

    path
}
