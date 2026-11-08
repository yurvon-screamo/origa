use crate::domain::OrigaError;
use crate::stt::tokenizer::WhisperTokenizer;

pub const MAX_DECODE_TOKENS: usize = 220;

pub fn build_prompt_tokens(tokenizer: &WhisperTokenizer) -> Result<Vec<i64>, OrigaError> {
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

pub fn argmax_last_position(logits: &ort::value::Value) -> Result<i64, OrigaError> {
    let (shape, data): (&ort::value::Shape, &[f32]) =
        logits
            .try_extract_tensor()
            .map_err(|e| OrigaError::SttError {
                reason: format!("Extract logits: {:?}", e),
            })?;

    let seq_len = shape[1] as usize;
    let vocab_size = shape[2] as usize;
    if seq_len == 0 || vocab_size == 0 {
        return Err(OrigaError::SttError {
            reason: "Invalid logits shape".to_string(),
        });
    }
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

pub fn strip_trailing_repeats(text: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::{build_prompt_tokens, strip_trailing_repeats};
    use crate::domain::OrigaError;
    use crate::stt::tokenizer::WhisperTokenizer;
    use std::path::PathBuf;

    fn real_tokenizer() -> WhisperTokenizer {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .join("tauri")
            .join("resources")
            .join("whisper")
            .join("tokenizer.json");
        WhisperTokenizer::from_json_file(&path).expect("committed tokenizer parses")
    }

    fn synthetic_tokenizer(json: &str) -> WhisperTokenizer {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("tokenizer.json");
        std::fs::write(&path, json).expect("write fixture");
        WhisperTokenizer::from_json_file(&path).expect("valid fixture")
    }

    #[test]
    fn build_prompt_tokens_returns_the_four_transcription_prompt_ids() {
        // Arrange
        let tokenizer = real_tokenizer();
        let expected: Vec<i64> = [
            "<|startoftranscript|>",
            "<|ja|>",
            "<|transcribe|>",
            "<|notimestamps|>",
        ]
        .iter()
        .map(|name| tokenizer.token_to_id(name).expect("special token"))
        .collect();

        // Act
        let tokens = build_prompt_tokens(&tokenizer).expect("prompt tokens");

        // Assert
        assert_eq!(tokens, expected);
    }

    #[test]
    fn build_prompt_tokens_missing_language_token_errors() {
        // Arrange — vocab carries only one of the four prompt tokens.
        let tokenizer = synthetic_tokenizer(r#"{"model": {"vocab": {"<|ja|>": 3}}}"#);

        // Act
        let result = build_prompt_tokens(&tokenizer);

        // Assert
        let Err(OrigaError::SttError { reason }) = result else {
            panic!("expected a missing-token error");
        };
        assert!(reason.contains("Missing token"), "reason: {reason}");
    }

    #[test]
    fn strip_trailing_repeats_returns_input_when_shorter_than_three_chars() {
        assert_eq!(strip_trailing_repeats("ab"), "ab");
        assert_eq!(strip_trailing_repeats("a"), "a");
        assert_eq!(strip_trailing_repeats(""), "");
    }

    #[test]
    fn strip_trailing_repeats_returns_input_when_last_two_differ() {
        assert_eq!(strip_trailing_repeats("abc"), "abc");
        assert_eq!(strip_trailing_repeats("あいう"), "あいう");
    }

    #[test]
    fn strip_trailing_repeats_collapses_run_of_identical_trailing_chars() {
        assert_eq!(strip_trailing_repeats("abcddd"), "abc");
        assert_eq!(strip_trailing_repeats("あいいいい"), "あ");
    }

    #[test]
    fn strip_trailing_repeats_all_identical_collapses_to_empty() {
        assert_eq!(strip_trailing_repeats("aaaa"), "");
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod argmax_tests {
    use super::argmax_last_position;
    use crate::domain::OrigaError;

    fn logits_value(data: Vec<f32>, seq_len: usize, vocab_size: usize) -> ort::value::Value {
        let array =
            ndarray::Array3::from_shape_vec((1, seq_len, vocab_size), data).expect("logits shape");
        ort::value::Tensor::from_array(array)
            .expect("ort tensor")
            .into_dyn()
    }

    #[test]
    fn argmax_picks_the_highest_logit_of_the_last_position() {
        // Arrange — last position row peaks at index 3; earlier rows are noise.
        let mut data = vec![0.0f32; 2 * 5];
        data[0] = 5.0; // earlier position — must be ignored
        data[5 + 3] = 2.0;
        data[5 + 1] = 0.5;
        let value = logits_value(data, 2, 5);

        // Act
        let id = argmax_last_position(&value).expect("argmax");

        // Assert
        assert_eq!(id, 3);
    }

    #[test]
    fn argmax_ties_resolve_to_the_first_index() {
        let data = vec![0.0f32; 4]; // all equal → first index wins
        let value = logits_value(data, 1, 4);
        assert_eq!(argmax_last_position(&value).expect("argmax"), 0);
    }

    #[test]
    fn argmax_empty_sequence_errors() {
        let value = logits_value(Vec::new(), 0, 5);
        let Err(OrigaError::SttError { reason }) = argmax_last_position(&value) else {
            panic!("expected an invalid-shape error");
        };
        assert!(reason.contains("Invalid logits shape"), "reason: {reason}");
    }
}
