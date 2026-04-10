use std::collections::{HashMap, HashSet};
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
