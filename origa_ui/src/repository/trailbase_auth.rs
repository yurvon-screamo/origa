use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthProvider {
    Google,
    Yandex,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::Yandex => "yandex",
        }
    }
}

pub fn decode_jwt_claims(token: &str) -> Result<JwtClaims, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format".to_string());
    }

    let payload = parts[1];
    let padding_len = (4 - payload.len() % 4) % 4;
    let padded = if padding_len > 0 {
        let mut s = payload.to_string();
        for _ in 0..padding_len {
            s.push('=');
        }
        s
    } else {
        payload.to_string()
    };

    let decoded = base64_decode(&padded)?;
    let json_str =
        String::from_utf8(decoded).map_err(|e| format!("Invalid UTF-8 in JWT payload: {}", e))?;

    serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JWT claims: {}", e))
}

pub fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let input = input.replace('-', "+").replace('_', "/");
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = Vec::new();
    let chars_vec: Vec<char> = chars.chars().collect();

    let clean_input: String = input.chars().filter(|c| *c != '=').collect();

    for chunk in clean_input.as_bytes().chunks(4) {
        let mut acc: u32 = 0;
        let mut bits = 0;

        for &byte in chunk {
            if let Some(pos) = chars_vec.iter().position(|&c| c == byte as char) {
                acc = (acc << 6) | pos as u32;
                bits += 6;
            }
        }

        while bits >= 8 {
            bits -= 8;
            result.push((acc >> bits) as u8);
        }
    }

    Ok(result)
}

pub fn generate_pkce_verifier() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::rng();
    (0..64)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn generate_pkce_challenge(verifier: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(result)
}

pub fn urlencoding_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}
