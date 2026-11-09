use std::collections::{HashMap, HashSet};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

pub struct WhisperTokenizer {
    id_to_token: Vec<String>,
    special_token_ids: HashSet<i64>,
    token_to_id_map: HashMap<String, i64>,
    byte_decoder: HashMap<char, u8>,
}

impl WhisperTokenizer {
    #[cfg(target_arch = "wasm32")]
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        let json: serde_json::Value = serde_json::from_slice(data)
            .map_err(|e| format!("Failed to parse tokenizer JSON: {}", e))?;

        let (id_to_token, token_to_id_map) = build_vocab(&json)?;
        let special_token_ids = collect_special_ids(&json);
        let byte_decoder = build_byte_decoder();

        Ok(Self {
            id_to_token,
            special_token_ids,
            token_to_id_map,
            byte_decoder,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_json_file(path: &Path) -> Result<Self, String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read tokenizer: {}", e))?;
        let json: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse tokenizer JSON: {}", e))?;

        let (id_to_token, token_to_id_map) = build_vocab(&json)?;
        let special_token_ids = collect_special_ids(&json);
        let byte_decoder = build_byte_decoder();

        Ok(Self {
            id_to_token,
            special_token_ids,
            token_to_id_map,
            byte_decoder,
        })
    }

    pub fn decode(&self, tokens: &[i64]) -> String {
        let mut bytes = Vec::new();
        for &token_id in tokens {
            if self.special_token_ids.contains(&token_id) {
                continue;
            }
            let idx = token_id as usize;
            if idx >= self.id_to_token.len() {
                continue;
            }
            let text = &self.id_to_token[idx];
            if text.starts_with("<|") && text.ends_with("|>") {
                continue;
            }
            for ch in text.chars() {
                if let Some(&byte) = self.byte_decoder.get(&ch) {
                    bytes.push(byte);
                }
            }
        }
        String::from_utf8_lossy(&bytes).to_string()
    }

    pub fn token_to_id(&self, text: &str) -> Option<i64> {
        self.token_to_id_map.get(text).copied()
    }
}

fn build_vocab(json: &serde_json::Value) -> Result<(Vec<String>, HashMap<String, i64>), String> {
    let vocab = json["model"]["vocab"]
        .as_object()
        .ok_or("Missing model.vocab in tokenizer.json")?;

    let mut max_id = vocab
        .values()
        .filter_map(|v| v.as_u64())
        .max()
        .ok_or("Empty vocab")? as usize;

    if let Some(added) = json["added_tokens"].as_array() {
        for token in added {
            if let Some(id) = token["id"].as_u64() {
                max_id = max_id.max(id as usize);
            }
        }
    }

    let mut id_to_token = vec![String::new(); max_id + 1];
    let mut token_to_id_map = HashMap::new();

    for (text, id) in vocab {
        let id = id.as_u64().ok_or("Non-integer token ID")? as usize;
        id_to_token[id] = text.clone();
        token_to_id_map.insert(text.clone(), id as i64);
    }

    if let Some(added) = json["added_tokens"].as_array() {
        for token in added {
            let id = match token["id"].as_u64() {
                Some(id) => id as usize,
                None => continue,
            };
            let content = match token["content"].as_str() {
                Some(c) => c.to_string(),
                None => continue,
            };
            id_to_token[id] = content.clone();
            token_to_id_map.insert(content, id as i64);
        }
    }

    Ok((id_to_token, token_to_id_map))
}

fn collect_special_ids(json: &serde_json::Value) -> HashSet<i64> {
    let mut ids = HashSet::new();
    if let Some(added) = json["added_tokens"].as_array() {
        for token in added {
            if token["special"].as_bool() == Some(true) {
                if let Some(id) = token["id"].as_i64() {
                    ids.insert(id);
                }
            }
        }
    }
    ids
}

fn build_byte_decoder() -> HashMap<char, u8> {
    let encoder = bytes_to_unicode();
    let mut decoder = HashMap::new();
    for (byte, &ch) in encoder.iter().enumerate() {
        decoder.insert(ch, byte as u8);
    }
    decoder
}

fn bytes_to_unicode() -> Vec<char> {
    let mut bs: Vec<u8> = (33..=126).chain(161..=172).chain(174..=255).collect();
    let mut cs: Vec<u32> = bs.iter().map(|&b| b as u32).collect();
    let mut n = 0u32;
    for b in 0u8..=255 {
        if !bs.contains(&b) {
            bs.push(b);
            cs.push(256 + n);
            n += 1;
        }
    }
    let mut map = vec!['\0'; 256];
    for (b, c) in bs.iter().zip(cs.iter()) {
        map[*b as usize] = char::from_u32(*c).expect("bytes_to_unicode: invalid codepoint");
    }
    map
}

#[cfg(test)]
mod tests {
    use super::WhisperTokenizer;
    use std::path::PathBuf;

    /// The committed Whisper tokenizer (no model download involved).
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

    fn synthetic_tokenizer(json: &str) -> Result<WhisperTokenizer, String> {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("tokenizer.json");
        std::fs::write(&path, json).expect("write fixture");
        // The parsed tokenizer owns no reference to the temp file.
        WhisperTokenizer::from_json_file(&path)
    }

    #[test]
    fn prompt_special_tokens_resolve_to_unique_ids() {
        let tokenizer = real_tokenizer();
        let ids: Vec<i64> = [
            "<|startoftranscript|>",
            "<|ja|>",
            "<|transcribe|>",
            "<|notimestamps|>",
            "<|endoftext|>",
        ]
        .iter()
        .map(|name| tokenizer.token_to_id(name))
        .collect::<Option<_>>()
        .expect("prompt tokens must exist in the committed tokenizer");
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "special tokens share an id");
    }

    #[test]
    fn decode_skips_special_tokens_and_delimiters() {
        let tokenizer = real_tokenizer();
        let prompt: Vec<i64> = [
            "<|startoftranscript|>",
            "<|ja|>",
            "<|transcribe|>",
            "<|notimestamps|>",
        ]
        .iter()
        .map(|name| tokenizer.token_to_id(name).expect("special token"))
        .collect();
        assert_eq!(tokenizer.decode(&prompt), "");
    }

    #[test]
    fn decode_maps_ascii_token_to_plain_text() {
        // Printable ASCII bytes map 1:1 through the byte encoder, so the
        // vocab key equals the decoded text.
        let tokenizer = real_tokenizer();
        let id = tokenizer.token_to_id("world").expect("'world' in vocab");
        assert_eq!(tokenizer.decode(&[id]), "world");
    }

    #[test]
    fn decode_maps_multibyte_token_to_japanese_text() {
        // 'こんにちは' byte-encoded through the GPT-2 byte alphabet is a
        // single vocab entry — the full UTF-8 roundtrip through decode.
        let tokenizer = real_tokenizer();
        let key = "ãģĵãĤĵãģ«ãģ¡ãģ¯";
        let id = tokenizer.token_to_id(key).expect("encoded key in vocab");
        assert_eq!(tokenizer.decode(&[id]), "こんにちは");
    }

    #[test]
    fn decode_skips_out_of_range_ids() {
        let tokenizer = real_tokenizer();
        assert_eq!(tokenizer.decode(&[999_999]), "");
    }

    #[test]
    fn missing_vocab_section_errors() {
        let Err(msg) = synthetic_tokenizer(r#"{"model": {}}"#) else {
            panic!("expected parse failure");
        };
        assert!(msg.contains("Missing model.vocab"));
    }

    #[test]
    fn empty_vocab_errors() {
        let Err(msg) = synthetic_tokenizer(r#"{"model": {"vocab": {}}}"#) else {
            panic!("expected parse failure");
        };
        assert!(msg.contains("Empty vocab"));
    }

    #[test]
    fn non_integer_token_id_errors() {
        // A valid id keeps the vocab non-empty, the string id fails the entry loop.
        let Err(msg) = synthetic_tokenizer(r#"{"model": {"vocab": {"a": 0, "b": "x"}}}"#) else {
            panic!("expected parse failure");
        };
        assert!(msg.contains("Non-integer token ID"));
    }

    #[test]
    fn special_added_tokens_are_skipped_by_decode() {
        // Arrange — one special token in added_tokens, one regular token.
        let json = r#"{
            "model": {"vocab": {"hello": 0}},
            "added_tokens": [
                {"id": 10, "content": "<|endoftext|>", "special": true},
                {"id": 12, "content": "added_thing", "special": false}
            ]
        }"#;
        let tokenizer = synthetic_tokenizer(json).expect("valid fixture");

        // Assert — special id is dropped, added regular token decodes.
        assert_eq!(tokenizer.token_to_id("<|endoftext|>"), Some(10));
        assert_eq!(tokenizer.decode(&[10]), "");
        assert_eq!(tokenizer.token_to_id("added_thing"), Some(12));
        assert_eq!(tokenizer.decode(&[12]), "added_thing");
    }

    #[test]
    fn byte_encoded_vocab_key_roundtrips_to_utf8_text() {
        // Arrange — key "Ã©" is the byte-alphabet spelling of UTF-8 é (C3 A9).
        let json = r#"{"model": {"vocab": {"Ã©": 5}}}"#;
        let tokenizer = synthetic_tokenizer(json).expect("valid fixture");

        // Act + Assert
        assert_eq!(tokenizer.decode(&[5]), "é");
    }

    #[test]
    fn gaps_in_token_ids_decode_to_nothing() {
        // Arrange — ids 1..5 are allocated but have no content.
        let json = r#"{"model": {"vocab": {"a": 0, "b": 5}}}"#;
        let tokenizer = synthetic_tokenizer(json).expect("valid fixture");

        // Act + Assert
        assert_eq!(tokenizer.decode(&[2]), "");
        assert_eq!(tokenizer.decode(&[0, 2, 5]), "ab");
    }
}
