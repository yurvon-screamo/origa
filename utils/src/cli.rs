use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tokenizer")]
#[command(about = "Unified CLI for Japanese tokenization and OCR tools", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Tokenize Japanese text and extract vocabulary words
    Tokenize {
        /// Text to tokenize or path to file
        text: String,

        /// Read text from file
        #[arg(short, long)]
        file: bool,
    },

    /// Japanese OCR using NDLOCR-Lite models
    Ndlocr {
        /// Input image path
        #[arg(short, long)]
        input: PathBuf,

        /// Detector model path
        #[arg(long, default_value = "../ndlocr-lite/src/model/deim-s-1024x1024.onnx")]
        detector: PathBuf,

        /// Parseq 30 model path
        #[arg(
            long,
            default_value = "../ndlocr-lite/src/model/parseq-ndl-16x256-30-tiny-192epoch-tegaki3.onnx"
        )]
        rec30: PathBuf,

        /// Parseq 50 model path
        #[arg(
            long,
            default_value = "../ndlocr-lite/src/model/parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx"
        )]
        rec50: PathBuf,

        /// Parseq 100 model path
        #[arg(
            long,
            default_value = "../ndlocr-lite/src/model/parseq-ndl-16x768-100-tiny-165epoch-tegaki2.onnx"
        )]
        rec100: PathBuf,

        /// Vocabulary file path
        #[arg(long, default_value = "config/NDLmoji.txt")]
        vocab: PathBuf,
    },

    /// Batch process JSON files in well_known_set format
    TokenizeWellKnown {
        /// Path to directory or JSON file
        path: PathBuf,
    },

    /// Find vocabulary words missing from dictionary and generate translations
    FindMissing {
        /// Output path for the markdown report (default: missing_vocabulary.md in project root)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Auto-generate missing words with translations
        #[arg(short, long)]
        generate: bool,

        /// OpenAI API base URL
        #[arg(long, default_value = "http://10.2.11.6:8001/v1")]
        api_base: String,

        /// OpenAI API key
        #[arg(long, default_value = "none")]
        api_key: String,

        /// Number of concurrent translation requests
        #[arg(short = 'w', long, default_value = "32")]
        workers: usize,

        /// Chunk size for processing
        #[arg(long, default_value = "512")]
        chunk_size: usize,

        /// Only translate to Russian
        #[arg(long)]
        russian_only: bool,

        /// Only translate to English
        #[arg(long)]
        english_only: bool,
    },

    /// Build phrase dataset from transcriptions
    BuildPhraseDataset {
        /// Input JSON file with transcriptions
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for results
        #[arg(short, long)]
        output: PathBuf,

        /// Minimum vocabulary tokens per phrase
        #[arg(long, default_value = "2")]
        min_tokens: usize,
    },
}
