use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub email: Option<String>,
    exp: Option<i64>,
    iss: Option<String>,
}

impl JwtClaims {
    pub fn expires_at(&self, fallback: u64) -> u64 {
        self.exp
            .and_then(|e| u64::try_from(e).ok())
            .unwrap_or(fallback)
    }
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
    let decoded = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|e| format!("Failed to decode JWT payload: {}", e))?;
    let json_str =
        String::from_utf8(decoded).map_err(|e| format!("Invalid UTF-8 in JWT payload: {}", e))?;

    let claims: JwtClaims = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse JWT claims: {}", e))?;

    if let Some(exp) = claims.exp {
        let now = chrono::Utc::now().timestamp();
        if now >= exp {
            return Err("Token expired".to_string());
        }
    }

    let iss = claims
        .iss
        .as_deref()
        .ok_or_else(|| "Missing issuer claim".to_string())?;
    let is_trusted = iss == "trailbase" || iss == "https://origa.uwuwu.net";
    if !is_trusted {
        return Err(format!("Invalid issuer: {}", iss));
    }

    Ok(claims)
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
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(result)
}

pub fn urlencoding_decode(s: &str) -> String {
    urlencoding::decode(s)
        .map(|cow| cow.into_owned())
        .unwrap_or_else(|_| s.to_string())
}
