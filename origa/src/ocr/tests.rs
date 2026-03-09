use crate::ocr::{JapaneseOCRModel, ModelConfig, ModelFiles};
use image::open;
use std::path::PathBuf;
use tokio::fs;

async fn get_file_bytes(url: &str, filename: &str) -> Vec<u8> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let cache_dir = PathBuf::from(manifest_dir)
        .join("target")
        .join("ocr_models");
    let file_path = cache_dir.join(filename);

    if file_path.exists() {
        return fs::read(&file_path)
            .await
            .expect("Failed to read cached model file");
    }

    fs::create_dir_all(&cache_dir)
        .await
        .expect("Failed to create cache directory");
    println!("Downloading {} from {}...", filename, url);

    let response = reqwest::get(url)
        .await
        .expect("Failed to download model file");
    let bytes = response
        .bytes()
        .await
        .expect("Failed to get model file bytes")
        .to_vec();

    fs::write(&file_path, &bytes)
        .await
        .expect("Failed to write model file to cache");
    bytes
}

#[tokio::test]
async fn test_japanese_ocr_e2e() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();

    let config = ModelConfig::default();

    let encoder = get_file_bytes(
        &config.ocr_model_file_url("encoder_model.onnx"),
        "encoder_model.onnx",
    )
    .await;
    let decoder = get_file_bytes(
        &config.ocr_model_file_url("decoder_model.onnx"),
        "decoder_model.onnx",
    )
    .await;
    let tokenizer = get_file_bytes(
        &config.ocr_model_file_url("tokenizer.json"),
        "tokenizer.json",
    )
    .await;
    let layout = get_file_bytes(
        &config.layout_model_file_url(),
        "doclayout_yolo_docstructbench_imgsz1024.onnx",
    )
    .await;

    let model_files = ModelFiles {
        encoder,
        decoder,
        tokenizer,
        layout_model: layout,
    };

    let mut model = JapaneseOCRModel::from_model_files(model_files)
        .expect("Failed to initialize JapaneseOCRModel");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let img_path = PathBuf::from(manifest_dir).join("src/ocr/ocr_example.jpg");
    let img = open(img_path).expect("Failed to open ocr_example.jpg");

    let result = model.run(&img).expect("Failed to run OCR");

    println!("OCR Result:\n{}", result);

    // Verify some text from the image
    assert!(
        result.contains("もんだい"),
        "Result should contain 'もんだい'"
    );
    assert!(result.contains("トイレ"), "Result should contain 'トイレ'");
    assert!(result.contains("山田"), "Result should contain '山田'");
}
