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

    // Load layout model from local directory
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = PathBuf::from(manifest_dir).parent().unwrap().to_path_buf();

    let layout_paths = [
        workspace_root
            .join("origa_ui")
            .join("public")
            .join("yolo")
            .join("layout.safetensors"),
        workspace_root.join(".yolo").join("model.safetensors"),
    ];

    let mut layout = None;
    for path in &layout_paths {
        if path.exists() {
            println!("Loading layout model from: {:?}", path);
            layout = Some(fs::read(path).await.expect("Failed to read layout model"));
            break;
        }
    }

    let layout = layout.expect(
        "Failed to find layout model. Please run scripts/download_yolo.py or ensure .yolo/model.safetensors exists.",
    );

    let model_files = ModelFiles {
        encoder,
        decoder,
        tokenizer,
        layout_model: layout,
    };

    let mut model = JapaneseOCRModel::from_model_files(model_files)
        .expect("Failed to initialize JapaneseOCRModel");

    let img_path = PathBuf::from(manifest_dir).join("src/ocr/ocr_example.jpg");
    let img = open(img_path).expect("Failed to open ocr_example.jpg");

    let result = model.run(&img).expect("Failed to run OCR");

    println!("OCR Result:\n{}", result);

    // Verify some text from the image
    assert!(result.contains("れんしゅう"), "should contain 'れんしゅう'");
    assert!(result.contains("もんだい"), "should contain 'もんだい'");
    assert!(result.contains("何を"), "should contain '何を'");
    assert!(result.contains("入れますか"), "should contain '入れますка'");
    assert!(result.contains("いちばん"), "should contain 'いちばん'");
    assert!(result.contains("えらんで"), "should contain 'えらんで'");
    assert!(result.contains("ください"), "should contain 'ください'");
    assert!(result.contains("トイレ"), "should contain 'トイレ'");
    assert!(result.contains("行って"), "should contain '行って'");
    assert!(result.contains("電車"), "should contain '電車'");
    assert!(result.contains("しまった"), "should contain 'しまった'");
    assert!(result.contains("すみません"), "should contain 'すみません'");
    assert!(result.contains("おそく"), "should contain 'おそく'");
    assert!(result.contains("なって"), "should contain 'なって'");
    assert!(result.contains("ずいぶん"), "should contain 'ずいぶん'");
    assert!(
        result.contains("待ちましたか"),
        "should contain '待ちましたか'"
    );
    assert!(result.contains("いいえ"), "should contain 'いいえ'");
    assert!(result.contains("わたしも"), "should contain 'わたしも'");
    assert!(result.contains("今"), "should contain '今'");
    assert!(result.contains("ところ"), "should contain 'ところ'");
    assert!(result.contains("来る"), "should contain '来る'");
    assert!(result.contains("来ている"), "should contain '来ている'");
    assert!(result.contains("来て"), "should contain '来て'");
    assert!(result.contains("来た"), "should contain '来た'");
    assert!(result.contains("山田"), "should contain '山田'");
    assert!(result.contains("もしもし"), "should contain 'もしもし'");
    assert!(result.contains("こちら"), "should contain 'こちら'");
    assert!(result.contains("スミス"), "should contain 'スミス'");
    assert!(
        result.contains("おねがいします"),
        "should contain 'おねがいします'"
    );
    assert!(result.contains("田中"), "should contain '田中'");
    assert!(result.contains("会議"), "should contain '会議'");
    assert!(result.contains("あいだ"), "should contain 'あいだ'");
    assert!(result.contains("ちゅう"), "should contain 'ちゅう'");
    assert!(result.contains("なか"), "should contain 'なか'");
    assert!(result.contains("じゅう"), "should contain 'じゅう'");
    assert!(result.contains("入る"), "should contain '入る'");
    assert!(result.contains("どれですか"), "should contain 'どれですか'");
}
