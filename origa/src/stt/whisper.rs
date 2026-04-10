use std::path::Path;
use std::sync::Mutex;

use crate::domain::OrigaError;
use ndarray::Array2;
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::Value;

pub use super::audio::load_wav;
pub use super::tokenizer::WhisperTokenizer;

const MAX_DECODE_TOKENS: usize = 220;

pub struct WhisperTranscriber {
    encoder_session: Mutex<Session>,
    decoder_session: Mutex<Session>,
    tokenizer: WhisperTokenizer,
}

impl WhisperTranscriber {
    pub fn new(model_dir: &Path) -> Result<Self, OrigaError> {
        tracing::info!(dir = ?model_dir, "Loading Whisper model");

        let encoder_bytes = std::fs::read(model_dir.join("encoder_model.onnx")).map_err(|e| {
            OrigaError::SttError {
                reason: format!("Read encoder: {}", e),
            }
        })?;
        let decoder_bytes = std::fs::read(model_dir.join("decoder_model.onnx")).map_err(|e| {
            OrigaError::SttError {
                reason: format!("Read decoder: {}", e),
            }
        })?;

        let encoder_session = create_session(&encoder_bytes, "encoder")?;
        let decoder_session = create_session(&decoder_bytes, "decoder")?;

        tracing::info!(inputs = ?encoder_session.inputs(), outputs = ?encoder_session.outputs(), "Encoder");
        tracing::info!(inputs = ?decoder_session.inputs(), outputs = ?decoder_session.outputs(), "Decoder");

        let tokenizer = WhisperTokenizer::from_json_file(&model_dir.join("tokenizer.json"))
            .map_err(|reason| OrigaError::SttError { reason })?;

        Ok(Self {
            encoder_session: Mutex::new(encoder_session),
            decoder_session: Mutex::new(decoder_session),
            tokenizer,
        })
    }

    pub fn transcribe(&self, wav_path: &Path) -> Result<String, OrigaError> {
        tracing::info!(path = ?wav_path, "Transcribing audio");

        let samples = load_wav(wav_path).map_err(|reason| OrigaError::SttError { reason })?;
        let mel = super::mel_spectrogram::compute_mel_spectrogram(&samples)
            .map_err(|reason| OrigaError::SttError { reason })?;

        let hidden_states = run_encoder(&self.encoder_session, &mel)?;
        let tokens = decode_autoregressive(&self.decoder_session, &self.tokenizer, &hidden_states)?;
        let text = self.tokenizer.decode(&tokens);
        let text = strip_trailing_repeats(&text);

        tracing::info!(text = %text, "Transcription complete");
        Ok(text)
    }
}

fn create_session(model_bytes: &[u8], label: &str) -> Result<Session, OrigaError> {
    let temp_dir = tempfile::tempdir().map_err(|e| OrigaError::SttError {
        reason: format!("Temp dir for {}: {}", label, e),
    })?;
    let path = temp_dir.path().join(format!("{}.onnx", label));
    std::fs::write(&path, model_bytes).map_err(|e| OrigaError::SttError {
        reason: format!("Write {}: {}", label, e),
    })?;

    let session = Session::builder()
        .map_err(|e| OrigaError::SttError {
            reason: format!("Builder {}: {:?}", label, e),
        })?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| OrigaError::SttError {
            reason: format!("OptLevel {}: {:?}", label, e),
        })?
        .commit_from_file(&path)
        .map_err(|e| OrigaError::SttError {
            reason: format!("Load {}: {:?}", label, e),
        })?;

    std::mem::forget(temp_dir);
    Ok(session)
}

fn run_encoder(
    session: &Mutex<Session>,
    mel: &ndarray::Array3<f32>,
) -> Result<ndarray::Array3<f32>, OrigaError> {
    let mel_tensor = Value::from_array(mel.clone()).map_err(|e| OrigaError::SttError {
        reason: format!("Mel tensor: {:?}", e),
    })?;

    let mut guard = session.lock().map_err(|e| OrigaError::SttError {
        reason: format!("Encoder lock: {:?}", e),
    })?;
    let outputs =
        guard
            .run(ort::inputs!["mel" => mel_tensor])
            .map_err(|e| OrigaError::SttError {
                reason: format!("Encoder run: {:?}", e),
            })?;

    let (shape, data): (&ort::value::Shape, &[f32]) =
        outputs[0]
            .try_extract_tensor()
            .map_err(|e| OrigaError::SttError {
                reason: format!("Extract encoder: {:?}", e),
            })?;

    ndarray::Array3::from_shape_vec(
        (shape[0] as usize, shape[1] as usize, shape[2] as usize),
        data.to_vec(),
    )
    .map_err(|e| OrigaError::SttError {
        reason: format!("Encoder shape: {:?}", e),
    })
}

fn build_prompt_tokens(tokenizer: &WhisperTokenizer) -> Result<Vec<i64>, OrigaError> {
    let lookup = |name: &str| {
        tokenizer
            .token_to_id(name)
            .ok_or_else(|| OrigaError::SttError {
                reason: format!("Missing token: {}", name),
            })
    };

    Ok(vec![
        lookup("<|startoftranscript|>")?,
        lookup("<|ja|>")?,
        lookup("<|transcribe|>")?,
        lookup("<|notimestamps|>")?,
    ])
}

fn decode_autoregressive(
    decoder_session: &Mutex<Session>,
    tokenizer: &WhisperTokenizer,
    hidden_states: &ndarray::Array3<f32>,
) -> Result<Vec<i64>, OrigaError> {
    let mut tokens = build_prompt_tokens(tokenizer)?;
    let eos_id = tokenizer
        .token_to_id("<|endoftranscript|>")
        .or_else(|| tokenizer.token_to_id(""))
        .ok_or_else(|| OrigaError::SttError {
            reason: "Missing EOS token".into(),
        })?;

    for _ in 0..MAX_DECODE_TOKENS {
        let input_array =
            Array2::from_shape_vec((1, tokens.len()), tokens.clone()).map_err(|e| {
                OrigaError::SttError {
                    reason: format!("Input IDs: {:?}", e),
                }
            })?;
        let input_tensor = Value::from_array(input_array).map_err(|e| OrigaError::SttError {
            reason: format!("Input tensor: {:?}", e),
        })?;
        let hidden_tensor =
            Value::from_array(hidden_states.clone()).map_err(|e| OrigaError::SttError {
                reason: format!("Hidden tensor: {:?}", e),
            })?;

        let new_token = {
            let mut guard = decoder_session.lock().map_err(|e| OrigaError::SttError {
                reason: format!("Decoder lock: {:?}", e),
            })?;
            let outputs = guard
                .run(ort::inputs![
                    "input_ids" => input_tensor,
                    "encoder_hidden_states" => hidden_tensor
                ])
                .map_err(|e| OrigaError::SttError {
                    reason: format!("Decoder run: {:?}", e),
                })?;

            argmax_last_position(&outputs[0])?
        };

        if new_token == eos_id {
            break;
        }
        tokens.push(new_token);
    }

    Ok(tokens)
}

fn argmax_last_position(logits: &Value) -> Result<i64, OrigaError> {
    let (shape, data): (&ort::value::Shape, &[f32]) =
        logits
            .try_extract_tensor()
            .map_err(|e| OrigaError::SttError {
                reason: format!("Extract logits: {:?}", e),
            })?;

    let seq_len = shape[1] as usize;
    let vocab_size = shape[2] as usize;
    let offset = (seq_len - 1) * vocab_size;

    let mut best_id = 0i64;
    let mut best_val = f32::NEG_INFINITY;
    for (i, &val) in data[offset..].iter().enumerate() {
        if val > best_val {
            best_val = val;
            best_id = i as i64;
        }
    }

    Ok(best_id)
}

pub(crate) fn strip_trailing_repeats(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 3 {
        return text.to_string();
    }
    let last = chars[chars.len() - 1];
    let second_last = chars[chars.len() - 2];
    if last != second_last {
        return text.to_string();
    }
    let cutoff = chars
        .iter()
        .rposition(|&c| c != last)
        .map(|pos| pos + 1)
        .unwrap_or(0);
    chars[..cutoff].iter().collect()
}
