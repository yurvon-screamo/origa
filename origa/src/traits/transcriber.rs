use std::future::Future;

use crate::domain::OrigaError;

/// Port for speech-to-text engines. The transcription use case owns audio
/// loading and error mapping; the engine owns mel/inference/decoding.
/// Implemented by `stt::WhisperTranscriber` (native and WASM); tests install
/// fakes to exercise the use case without ONNX models.
pub trait Transcriber {
    fn transcribe_samples(
        &self,
        samples: &[f32],
    ) -> impl Future<Output = Result<String, OrigaError>>;
}
