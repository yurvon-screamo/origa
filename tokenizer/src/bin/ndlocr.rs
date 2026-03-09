use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tokenizer::ocr::OcrEngine;

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

fn main() -> Result<()> {
    let args = Cli::parse();

    let img = image::open(&args.input)
        .with_context(|| format!("Failed to open image: {:?}", args.input))?;

    let engine = OcrEngine::new(
        &args.detector,
        (&args.rec30, &args.rec50, &args.rec100),
        &args.vocab,
    )?;

    let text = engine.recognize(&img)?;

    println!("{}", text);

    Ok(())
}
