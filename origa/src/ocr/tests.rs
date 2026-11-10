use crate::ocr::{JapaneseOCRModel, ModelFiles};
use image::open;
use tokio::fs;

#[tokio::test]
async fn japanese_ocr_e2e() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();

    let ndlocr_model_dir = crate::corpus::cdn_dir().join("ndlocr");

    if !ndlocr_model_dir.join("deim.onnx").exists() {
        eprintln!("Skipping OCR test: model files not available in CI");
        return;
    }

    let deim = fs::read(ndlocr_model_dir.join("deim.onnx"))
        .await
        .expect("Failed to read deim model.");

    let parseq30 = fs::read(ndlocr_model_dir.join("parseq-30.onnx"))
        .await
        .expect("Failed to read parseq30 model.");

    let parseq50 = fs::read(ndlocr_model_dir.join("parseq-50.onnx"))
        .await
        .expect("Failed to read parseq50 model.");

    let parseq100 = fs::read(ndlocr_model_dir.join("parseq-100.onnx"))
        .await
        .expect("Failed to read parseq100 model.");

    let vocab = fs::read(ndlocr_model_dir.join("vocab.txt"))
        .await
        .expect("Failed to read vocab file. Ensure tokenizer submodule is initialized.");

    let model_files = ModelFiles {
        deim,
        parseq30,
        parseq50,
        parseq100,
        vocab,
    };

    let model = JapaneseOCRModel::from_model_files(model_files)
        .expect("Failed to initialize JapaneseOCRModel");

    let img_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ocr/ocr_example.jpg");
    let img = open(img_path).expect("Failed to open ocr_example.jpg");

    let result = model.run(&img).expect("Failed to run OCR");

    println!("OCR Result:\n{}", result);

    assert!(result.contains("れんしゅう"), "should contain 'れんしゅう'");
    assert!(result.contains("もんだい"), "should contain 'もんだい'");
    assert!(result.contains("何を"), "should contain '何を'");
    assert!(result.contains("入れますか"), "should contain '入れますか'");
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

#[test]
fn ort_error_maps_to_ocr_error_with_the_original_reason() {
    // Arrange
    let ort_error = ort::Error::new("boom");

    // Act
    let mapped: crate::domain::OrigaError = ort_error.into();

    // Assert
    let crate::domain::OrigaError::OcrError { reason } = mapped else {
        panic!("expected OcrError");
    };
    assert!(reason.contains("boom"), "reason: {reason}");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn deim_detector_new_missing_model_file_errors() {
    // Arrange — a path with no ONNX model behind it.
    let missing = std::env::temp_dir().join("origa-no-such-deim.onnx");

    // Act
    let result = crate::ocr::DeimDetector::new(&missing);

    // Assert
    let Err(crate::domain::OrigaError::OcrError { reason }) = result else {
        panic!("expected a model-loading failure");
    };
    assert!(
        reason.contains("Failed to load DEIM model"),
        "reason: {reason}"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn parseq_recognizer_new_missing_model_file_errors() {
    // Arrange
    let missing = std::env::temp_dir().join("origa-no-such-parseq.onnx");
    let vocab = crate::ocr::Vocabulary::from_bytes("あ\nい\n".as_bytes()).expect("vocab");

    // Act
    let result = crate::ocr::ParseqRecognizer::new(&missing, &vocab, 320);

    // Assert
    let Err(crate::domain::OrigaError::OcrError { reason }) = result else {
        panic!("expected a model-loading failure");
    };
    assert!(
        reason.contains("Failed to load PARSeq model"),
        "reason: {reason}"
    );
}
