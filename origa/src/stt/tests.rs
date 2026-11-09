//! Whisper pipeline smoke tests. The ONNX models are not part of the repo
//! (gitignored under tauri/resources/whisper) — model-dependent tests bail
//! out when the files are absent, so CI coverage runs stay green while the
//! tests stay alive on machines that have the models.

use crate::stt::whisper::WhisperTranscriber;
use std::path::PathBuf;

fn whisper_model_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("tauri")
        .join("resources")
        .join("whisper")
}

fn models_available() -> bool {
    whisper_model_dir().join("encoder_model.onnx").exists()
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn whisper_transcriber_loads_models() {
    if !models_available() {
        println!(
            "Skipping: whisper models not found at {:?}",
            whisper_model_dir()
        );
        return;
    }

    let transcriber = WhisperTranscriber::new(&whisper_model_dir());
    assert!(
        transcriber.is_ok(),
        "Failed to create WhisperTranscriber: {:?}",
        transcriber.err()
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn whisper_transcriber_new_missing_model_dir_errors_with_read_encoder() {
    let missing = std::env::temp_dir().join("origa-no-such-whisper-dir");
    let result = WhisperTranscriber::new(&missing);
    let Err(crate::domain::OrigaError::SttError { reason }) = result else {
        panic!("expected a model-loading failure");
    };
    assert!(reason.contains("Read encoder"), "reason: {reason}");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn whisper_transcribe_audio() {
    let wav_path = whisper_model_dir()
        .parent()
        .unwrap()
        .join("standard_sample1.wav");

    if !models_available() || !wav_path.exists() {
        println!("Skipping: whisper models or test audio not found");
        return;
    }

    let transcriber = WhisperTranscriber::new(&whisper_model_dir()).unwrap();
    let text = transcriber.transcribe(&wav_path);
    assert!(text.is_ok(), "Transcription failed: {:?}", text.err());

    let result = text.unwrap();
    assert!(!result.is_empty(), "Transcription should not be empty");
    println!("Transcription result: {}", result);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn audio_load_wav_sample() {
    // Arrange — generate the sample in a temp file (hound), 8 kHz mono.
    let dir = tempfile::tempdir().expect("temp dir");
    let wav_path = dir.path().join("sample.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&wav_path, spec).expect("wav writer");
    for i in 0..1600i16 {
        writer.write_sample((i % 100) * 300).expect("write sample");
    }
    writer.finalize().expect("finalize");

    // Act
    let samples = crate::stt::load_wav(&wav_path);

    // Assert
    let result = samples.expect("Failed to load WAV");
    assert_eq!(
        result.len(),
        480_000,
        "Should be padded/trimmed to 30s at 16kHz"
    );
}
