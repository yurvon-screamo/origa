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
