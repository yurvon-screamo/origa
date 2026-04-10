use std::cell::Cell;

use super::common::{
    MAX_DECODE_TOKENS, argmax_last_position, build_prompt_tokens, strip_trailing_repeats,
};
use crate::domain::OrigaError;
use crate::stt::tokenizer::WhisperTokenizer;
use futures::lock::Mutex;
use ort::session::RunOptions;
use ort_web::ValueExt;

thread_local! {
    static ORT_INITIALIZED: Cell<bool> = const { Cell::new(false) };
}

async fn ensure_ort_initialized() -> Result<(), OrigaError> {
    let already_init = ORT_INITIALIZED.with(|c| c.get());
    if already_init {
        return Ok(());
    }

    let api = ort_web::api(ort_web::FEATURE_NONE)
        .await
        .map_err(|e| OrigaError::SttError {
            reason: format!("Failed to get ort API: {:?}", e),
        })?;
    ort::set_api(api);

    ORT_INITIALIZED.with(|c| c.set(true));
    Ok(())
}

pub struct WhisperTranscriber {
    encoder_session: Mutex<ort::session::Session>,
    decoder_session: Mutex<ort::session::Session>,
    tokenizer: WhisperTokenizer,
}

impl WhisperTranscriber {
    pub async fn new(
        encoder_bytes: &[u8],
        decoder_bytes: &[u8],
        tokenizer_bytes: &[u8],
    ) -> Result<Self, OrigaError> {
        ensure_ort_initialized().await?;

        tracing::info!("Loading Whisper model for WASM");

        let encoder_session = {
            let mut builder =
                ort::session::Session::builder().map_err(|e| OrigaError::SttError {
                    reason: format!("Encoder builder: {:?}", e),
                })?;
            builder
                .commit_from_memory(encoder_bytes)
                .await
                .map_err(|e| OrigaError::SttError {
                    reason: format!("Load encoder: {:?}", e),
                })?
        };

        let decoder_session = {
            let mut builder =
                ort::session::Session::builder().map_err(|e| OrigaError::SttError {
                    reason: format!("Decoder builder: {:?}", e),
                })?;
            builder
                .commit_from_memory(decoder_bytes)
                .await
                .map_err(|e| OrigaError::SttError {
                    reason: format!("Load decoder: {:?}", e),
                })?
        };

        tracing::info!(
            inputs = ?encoder_session.inputs(),
            outputs = ?encoder_session.outputs(),
            "Encoder loaded"
        );
        tracing::info!(
            inputs = ?decoder_session.inputs(),
            outputs = ?decoder_session.outputs(),
            "Decoder loaded"
        );

        let tokenizer = WhisperTokenizer::from_bytes(tokenizer_bytes)
            .map_err(|reason| OrigaError::SttError { reason })?;

        Ok(Self {
            encoder_session: Mutex::new(encoder_session),
            decoder_session: Mutex::new(decoder_session),
            tokenizer,
        })
    }

    pub async fn transcribe_from_samples(&self, samples: &[f32]) -> Result<String, OrigaError> {
        tracing::info!(samples_len = samples.len(), "Transcribing audio (WASM)");

        let mel = super::mel_spectrogram::compute_mel_spectrogram(samples)
            .map_err(|reason| OrigaError::SttError { reason })?;

        let hidden_states = self.run_encoder(&mel).await?;
        let tokens = self.decode_autoregressive(&hidden_states).await?;
        let text = self.tokenizer.decode(&tokens);
        let text = strip_trailing_repeats(&text);

        tracing::info!(text = %text, "Transcription complete (WASM)");
        Ok(text)
    }

    async fn run_encoder(
        &self,
        mel: &ndarray::Array3<f32>,
    ) -> Result<ndarray::Array3<f32>, OrigaError> {
        let shape: Vec<usize> = mel.shape().to_vec();
        let data: Vec<f32> = mel.clone().into_raw_vec_and_offset().0;
        let mel_tensor = ort::value::Tensor::from_array((shape, data))
            .map_err(|e| OrigaError::SttError {
                reason: format!("Mel tensor: {:?}", e),
            })?
            .into_dyn();

        let mut session = self.encoder_session.lock().await;
        let run_options = RunOptions::new().map_err(|e| OrigaError::SttError {
            reason: format!("Run options: {:?}", e),
        })?;

        let mut outputs = session
            .run_async(ort::inputs!["mel" => mel_tensor], &run_options)
            .await
            .map_err(|e| OrigaError::SttError {
                reason: format!("Encoder run: {:?}", e),
            })?;

        for (_name, mut output) in outputs.iter_mut() {
            output
                .sync(ort_web::SyncDirection::Rust)
                .await
                .map_err(|e| OrigaError::SttError {
                    reason: format!("Encoder sync: {:?}", e),
                })?;
        }

        let (out_shape, out_data): (&ort::value::Shape, &[f32]) =
            outputs[0]
                .try_extract_tensor()
                .map_err(|e| OrigaError::SttError {
                    reason: format!("Extract encoder: {:?}", e),
                })?;

        ndarray::Array3::from_shape_vec(
            (
                out_shape[0] as usize,
                out_shape[1] as usize,
                out_shape[2] as usize,
            ),
            out_data.to_vec(),
        )
        .map_err(|e| OrigaError::SttError {
            reason: format!("Encoder shape: {:?}", e),
        })
    }

    async fn decode_autoregressive(
        &self,
        hidden_states: &ndarray::Array3<f32>,
    ) -> Result<Vec<i64>, OrigaError> {
        let mut tokens = build_prompt_tokens(&self.tokenizer)?;
        let eos_id = self
            .tokenizer
            .token_to_id("<|endoftranscript|>")
            .ok_or_else(|| OrigaError::SttError {
                reason: "Missing EOS token".into(),
            })?;

        for _ in 0..MAX_DECODE_TOKENS {
            let input_shape = vec![1usize, tokens.len()];
            let input_data: Vec<i64> = tokens.clone();
            let input_tensor = ort::value::Tensor::from_array((input_shape, input_data))
                .map_err(|e| OrigaError::SttError {
                    reason: format!("Input tensor: {:?}", e),
                })?
                .into_dyn();

            let hidden_shape: Vec<usize> = hidden_states.shape().to_vec();
            let hidden_data: Vec<f32> = hidden_states.clone().into_raw_vec_and_offset().0;
            let hidden_tensor = ort::value::Tensor::from_array((hidden_shape, hidden_data))
                .map_err(|e| OrigaError::SttError {
                    reason: format!("Hidden tensor: {:?}", e),
                })?
                .into_dyn();

            let new_token = {
                let mut session = self.decoder_session.lock().await;
                let run_options = RunOptions::new().map_err(|e| OrigaError::SttError {
                    reason: format!("Run options: {:?}", e),
                })?;

                let mut outputs = session
                    .run_async(
                        ort::inputs![
                            "input_ids" => input_tensor,
                            "encoder_hidden_states" => hidden_tensor
                        ],
                        &run_options,
                    )
                    .await
                    .map_err(|e| OrigaError::SttError {
                        reason: format!("Decoder run: {:?}", e),
                    })?;

                for (_name, mut output) in outputs.iter_mut() {
                    output
                        .sync(ort_web::SyncDirection::Rust)
                        .await
                        .map_err(|e| OrigaError::SttError {
                            reason: format!("Decoder sync: {:?}", e),
                        })?;
                }

                argmax_last_position(&outputs[0])?
            };

            if new_token == eos_id {
                break;
            }
            tokens.push(new_token);
        }

        Ok(tokens)
    }
}
