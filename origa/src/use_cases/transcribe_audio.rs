use crate::domain::OrigaError;
use crate::stt::WhisperTranscriber;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
use tracing::info;

pub struct TranscribeAudioUseCase;

impl Default for TranscribeAudioUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscribeAudioUseCase {
    pub fn new() -> Self {
        Self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn execute_with_path(
        &self,
        model: &WhisperTranscriber,
        wav_path: &std::path::Path,
    ) -> Result<String, OrigaError> {
        info!(path = ?wav_path, "Executing TranscribeAudioUseCase");
        model.transcribe(wav_path)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn execute(
        &self,
        model: Rc<WhisperTranscriber>,
        audio_bytes: &[u8],
    ) -> Result<String, OrigaError> {
        info!(
            bytes_len = audio_bytes.len(),
            "Executing TranscribeAudioUseCase (WASM)"
        );

        let samples = crate::stt::load_audio_bytes(audio_bytes)
            .map_err(|reason| OrigaError::SttError { reason })?;

        model.transcribe_from_samples(&samples).await
    }
}
