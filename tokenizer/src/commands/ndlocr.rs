use origa::domain::OrigaError;
use origa::ocr::{JapaneseOCRModel, ModelFiles};
use std::fs;
use std::path::PathBuf;

/// Loads a model file and returns an error with context if it fails
fn load_model_file(path: &PathBuf, model_name: &str) -> Result<Vec<u8>, OrigaError> {
    fs::read(path).map_err(|e| OrigaError::OcrError {
        reason: format!("Failed to read {}: {}", model_name, e),
    })
}

/// Runs Japanese OCR using NDLOCR-Lite models
pub fn run_ndlocr(
    input: PathBuf,
    detector: PathBuf,
    rec30: PathBuf,
    rec50: PathBuf,
    rec100: PathBuf,
    vocab: PathBuf,
) -> Result<(), OrigaError> {
    // Load image
    let img = image::open(&input).map_err(|e| OrigaError::OcrError {
        reason: format!("Failed to open image {:?}: {}", input, e),
    })?;

    // Load all model files
    let model_files = ModelFiles {
        deim: load_model_file(&detector, "detector model")?,
        parseq30: load_model_file(&rec30, "parseq30 model")?,
        parseq50: load_model_file(&rec50, "parseq50 model")?,
        parseq100: load_model_file(&rec100, "parseq100 model")?,
        vocab: load_model_file(&vocab, "vocabulary")?,
    };

    // Run OCR
    let model = JapaneseOCRModel::from_model_files(model_files)?;
    let text = model.run(&img)?;

    println!("{}", text);
    Ok(())
}
