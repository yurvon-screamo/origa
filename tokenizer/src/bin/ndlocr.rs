use clap::Parser;
use origa::domain::OrigaError;
use origa::ocr::{JapaneseOCRModel, ModelFiles};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ndlocr")]
#[command(about = "NDLOCR-Lite CLI - Japanese OCR", long_about = None)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(long, default_value = "../ndlocr-lite/src/model/deim-s-1024x1024.onnx")]
    detector: PathBuf,

    #[arg(
        long,
        default_value = "../ndlocr-lite/src/model/parseq-ndl-16x256-30-tiny-192epoch-tegaki3.onnx"
    )]
    rec30: PathBuf,

    #[arg(
        long,
        default_value = "../ndlocr-lite/src/model/parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx"
    )]
    rec50: PathBuf,

    #[arg(
        long,
        default_value = "../ndlocr-lite/src/model/parseq-ndl-16x768-100-tiny-165epoch-tegaki2.onnx"
    )]
    rec100: PathBuf,

    #[arg(long, default_value = "config/NDLmoji.txt")]
    vocab: PathBuf,
}

fn main() -> Result<(), OrigaError> {
    let args = Cli::parse();

    let img = image::open(&args.input).map_err(|e| OrigaError::OcrError {
        reason: format!("Failed to open image {:?}: {}", args.input),
    })?;

    let deim = std::fs::read(&args.detector).map_err(|e| OrigaError::OcrError {
        reason: format!("Failed to read detector model: {}", e),
    })?;
    let parseq30 = std::fs::read(&args.rec30).map_err(|e| OrigaError::OcrError {
        reason: format!("Failed to read parseq30 model: {}", e),
    })?;
    let parseq50 = std::fs::read(&args.rec50).map_err(|e| OrigaError::OcrError {
        reason: format!("Failed to read parseq50 model: {}", e),
    })?;
    let parseq100 = std::fs::read(&args.rec100).map_err(|e| OrigaError::OcrError {
        reason: format!("Failed to read parseq100 model: {}", e),
    })?;
    let vocab = std::fs::read(&args.vocab).map_err(|e| OrigaError::OcrError {
        reason: format!("Failed to read vocabulary: {}", e),
    })?;

    let model_files = ModelFiles {
        deim,
        parseq30,
        parseq50,
        parseq100,
        vocab,
    };

    let mut model = JapaneseOCRModel::from_model_files(model_files)?;
    let text = model.run(&img)?;

    println!("{}", text);
    Ok(())
}
