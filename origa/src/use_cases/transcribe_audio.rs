use crate::domain::OrigaError;
use crate::stt::load_audio_bytes;
#[cfg(not(target_arch = "wasm32"))]
use crate::stt::load_wav;
use crate::traits::Transcriber;
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
    pub async fn execute_with_path<T: Transcriber>(
        &self,
        engine: &T,
        wav_path: &std::path::Path,
    ) -> Result<String, OrigaError> {
        info!(path = ?wav_path, "Executing TranscribeAudioUseCase");
        let samples = load_wav(wav_path).map_err(|reason| OrigaError::SttError { reason })?;
        engine.transcribe_samples(&samples).await
    }

    pub async fn execute<T: Transcriber>(
        &self,
        engine: Rc<T>,
        audio_bytes: &[u8],
    ) -> Result<String, OrigaError> {
        info!(
            bytes_len = audio_bytes.len(),
            "Executing TranscribeAudioUseCase (WASM)"
        );

        let samples =
            load_audio_bytes(audio_bytes).map_err(|reason| OrigaError::SttError { reason })?;

        engine.transcribe_samples(&samples).await
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::TranscribeAudioUseCase;
    use crate::domain::OrigaError;
    use crate::traits::Transcriber;
    use std::cell::Cell;
    use std::future::Future;
    use std::path::Path;
    use std::rc::Rc;

    /// Recording fake over the engine port: counts calls, captures the
    /// samples handed over by the use case and returns a canned result.
    struct FakeTranscriber {
        calls: Cell<usize>,
        samples_len: Cell<usize>,
        samples_head: std::cell::RefCell<Vec<f32>>,
        result: Result<String, OrigaError>,
    }

    impl FakeTranscriber {
        fn returning(result: Result<String, OrigaError>) -> Self {
            Self {
                calls: Cell::new(0),
                samples_len: Cell::new(0),
                samples_head: std::cell::RefCell::new(Vec::new()),
                result,
            }
        }
    }

    impl Transcriber for FakeTranscriber {
        fn transcribe_samples(
            &self,
            samples: &[f32],
        ) -> impl Future<Output = Result<String, OrigaError>> {
            self.calls.set(self.calls.get() + 1);
            self.samples_len.set(samples.len());
            *self.samples_head.borrow_mut() = samples.iter().take(8).copied().collect();
            std::future::ready(self.result.clone())
        }
    }

    /// Writes a mono 16 kHz PCM16 fixture; the caller keeps the returned
    /// TempDir alive for the duration of the test — no /tmp leakage.
    fn wav_path_with_samples(samples: &[i16]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("input.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).expect("wav writer");
        for &s in samples {
            writer.write_sample(s).expect("write sample");
        }
        writer.finalize().expect("finalize");
        (dir, path)
    }

    fn engine_error() -> OrigaError {
        OrigaError::SttError {
            reason: "engine boom".to_string(),
        }
    }

    #[tokio::test]
    async fn execute_with_path_passes_decoded_samples_to_engine_and_returns_text() {
        // Arrange
        let raw: [i16; 4] = [100, -100, 3276, -3276];
        let (_dir, path) = wav_path_with_samples(&raw);
        let engine = FakeTranscriber::returning(Ok("こんにちは".to_string()));
        let use_case = TranscribeAudioUseCase::new();

        // Act
        let text = use_case.execute_with_path(&engine, &path).await.unwrap();

        // Assert
        assert_eq!(text, "こんにちは");
        assert_eq!(engine.calls.get(), 1);
        // load_wav pads/trims to 30s at 16kHz
        assert_eq!(engine.samples_len.get(), 480_000);
        let head = engine.samples_head.borrow();
        for (i, &expected) in raw.iter().enumerate() {
            let decoded = expected as f32 / 32768.0;
            assert!(
                (head[i] - decoded).abs() < 1e-6,
                "sample {i}: {} vs {decoded}",
                head[i]
            );
        }
    }

    #[tokio::test]
    async fn execute_with_path_missing_file_errors_without_calling_engine() {
        // Arrange
        let engine = FakeTranscriber::returning(Ok("unused".to_string()));
        let use_case = TranscribeAudioUseCase::new();

        // Act
        let result = use_case
            .execute_with_path(&engine, Path::new("/nonexistent/input.wav"))
            .await;

        // Assert
        assert!(matches!(result, Err(OrigaError::SttError { .. })));
        assert_eq!(engine.calls.get(), 0);
    }

    #[tokio::test]
    async fn execute_with_path_engine_error_is_returned_unchanged() {
        // Arrange
        let (_dir, path) = wav_path_with_samples(&[100, -100]);
        let engine = FakeTranscriber::returning(Err(engine_error()));
        let use_case = TranscribeAudioUseCase::new();

        // Act
        let result = use_case.execute_with_path(&engine, &path).await;

        // Assert
        assert_eq!(result.unwrap_err(), engine_error());
        assert_eq!(engine.calls.get(), 1);
    }

    #[tokio::test]
    async fn execute_rejects_non_wav_bytes_without_calling_engine() {
        // Arrange — keep a handle to the engine to assert it was not called.
        let engine = Rc::new(FakeTranscriber::returning(Ok("unused".to_string())));
        let handle = Rc::clone(&engine);
        let use_case = TranscribeAudioUseCase::new();

        // Act
        let result = use_case.execute(engine, b"not a wav file at all....").await;

        // Assert
        assert!(matches!(result, Err(OrigaError::SttError { .. })));
        assert_eq!(handle.calls.get(), 0);
    }

    #[tokio::test]
    async fn execute_passes_decoded_wav_bytes_to_engine_and_returns_text() {
        // Arrange — a minimal RIFF/WAVE payload with one PCM16 chunk.
        let raw: [i16; 3] = [1000, -1000, 500];
        let engine = Rc::new(FakeTranscriber::returning(Ok("テスト".to_string())));
        let engine_handle = engine.clone();
        let use_case = TranscribeAudioUseCase::new();

        // Act
        let text = use_case
            .execute(engine, &test_wav_bytes(&raw))
            .await
            .unwrap();

        // Assert
        assert_eq!(text, "テスト");
        assert_eq!(engine_handle.calls.get(), 1);
        // load_audio_bytes pads/trims to 30s at 16kHz
        assert_eq!(engine_handle.samples_len.get(), 480_000);
        let head = engine_handle.samples_head.borrow();
        for (i, &expected) in raw.iter().enumerate() {
            let decoded = expected as f32 / 32768.0;
            assert!(
                (head[i] - decoded).abs() < 1e-6,
                "sample {i}: {} vs {decoded}",
                head[i]
            );
        }
    }

    /// Builds minimal RIFF/WAVE bytes: fmt (PCM16 mono 16kHz) + data.
    fn test_wav_bytes(samples: &[i16]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        let data_len = (samples.len() * 2) as u32;
        // 4 (WAVE) + 8 + 16 (fmt) + 8 + data
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
        bytes.extend_from_slice(&16_000u32.to_le_bytes());
        bytes.extend_from_slice(&32_000u32.to_le_bytes()); // byte rate
        bytes.extend_from_slice(&2u16.to_le_bytes()); // block align
        bytes.extend_from_slice(&16u16.to_le_bytes()); // bits
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for &s in samples {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        bytes
    }
}
