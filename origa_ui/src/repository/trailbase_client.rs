use crate::repository::session::{TrailBaseSession, set_session};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use gloo_net::http::{Method, Request, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

const TRAILBASE_URL: &str = "https://origa-app.up.railway.app";
const REFRESH_THRESHOLD_SECONDS: u64 = 300;

#[derive(Debug, Deserialize)]
struct JwtClaims {
    sub: String,
    email: Option<String>,
}

fn decode_jwt_claims(token: &str) -> Result<JwtClaims, String> {
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

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
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

#[derive(Clone, Debug)]
pub struct TrailBaseClient {
    base_url: String,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Session expired, please login again")]
    SessionExpired,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
}

#[derive(Debug, Deserialize)]
struct AuthTokenResponse {
    auth_token: String,
    refresh_token: Option<String>,
    csrf_token: Option<String>,
}

impl TrailBaseClient {
    pub fn new() -> Self {
        Self {
            base_url: TRAILBASE_URL.to_string(),
        }
    }

    pub fn with_url(base_url: String) -> Self {
        Self { base_url }
    }

    fn current_timestamp() -> u64 {
        chrono::Utc::now().timestamp() as u64
    }

    fn should_refresh(expires_at: u64) -> bool {
        let now = Self::current_timestamp();
        now.saturating_add(REFRESH_THRESHOLD_SECONDS) >= expires_at
    }

    async fn fetch<T: Serialize>(
        &self,
        path: &str,
        method: Method,
        body: Option<&T>,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Response, AuthError> {
        let url = format!("{}{}", self.base_url, path);

        let request_builder = match method {
            Method::GET => Request::get(&url),
            Method::POST => Request::post(&url),
            Method::PUT => Request::put(&url),
            Method::DELETE => Request::delete(&url),
            Method::PATCH => Request::patch(&url),
            _ => {
                return Err(AuthError::ApiError(format!(
                    "Unsupported HTTP method: {:?}",
                    method
                )));
            }
        };

        let request_builder = if let Some(h) = headers {
            let mut builder = request_builder;
            for (key, value) in h {
                builder = builder.header(&key, &value);
            }
            builder
        } else {
            request_builder
        };

        let request = if let Some(json_body) = body {
            let json = serde_json::to_string(json_body)
                .map_err(|e| AuthError::ApiError(format!("Failed to serialize: {}", e)))?;
            request_builder
                .header("Content-Type", "application/json")
                .body(json)
                .map_err(|e| AuthError::ApiError(format!("Failed to build request: {}", e)))?
        } else {
            request_builder
                .build()
                .map_err(|e| AuthError::ApiError(format!("Failed to build request: {}", e)))?
        };

        request
            .send()
            .await
            .map_err(|e| AuthError::NetworkError(e.to_string()))
    }

    async fn json<T: DeserializeOwned>(response: Response) -> Result<T, AuthError> {
        response
            .json()
            .await
            .map_err(|e| AuthError::ApiError(format!("Failed to parse response: {}", e)))
    }

    pub fn generate_pkce_verifier() -> String {
        use rand::Rng;
        const CHARSET: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
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

    pub fn get_oauth_url(provider: &str, redirect_uri: &str, pkce_challenge: &str) -> String {
        let encoded_redirect = urlencoding::encode(redirect_uri);
        let encoded_challenge = urlencoding::encode(pkce_challenge);

        let scope = if provider == "oidc0" {
            "&scope=login:email%20login:info"
        } else {
            ""
        };

        format!(
            "{}/api/auth/v1/oauth/{}/login?redirect_uri={}&response_type=code&pkce_code_challenge={}{}",
            TRAILBASE_URL, provider, encoded_redirect, encoded_challenge, scope
        )
    }

    pub async fn exchange_auth_code_for_session(
        &self,
        code: &str,
        pkce_verifier: &str,
    ) -> Result<TrailBaseSession, AuthError> {
        #[derive(Serialize)]
        struct TokenRequest<'a> {
            authorization_code: &'a str,
            pkce_code_verifier: &'a str,
        }

        let response = self
            .fetch(
                "/api/auth/v1/token",
                Method::POST,
                Some(&TokenRequest {
                    authorization_code: code,
                    pkce_code_verifier: pkce_verifier,
                }),
                None,
            )
            .await?;

        if !response.ok() {
            return Err(AuthError::ApiError(format!(
                "Token exchange failed: {}",
                response.status_text()
            )));
        }

        let token_response: AuthTokenResponse = Self::json(response).await?;

        let claims = decode_jwt_claims(&token_response.auth_token)
            .map_err(|e| AuthError::ApiError(format!("Failed to decode JWT: {}", e)))?;

        let now = Self::current_timestamp();
        let expires_at = now.saturating_add(3600);

        let session = TrailBaseSession {
            auth_token: token_response.auth_token,
            refresh_token: token_response.refresh_token.unwrap_or_default(),
            email: claims.email.clone().unwrap_or_default(),
            auth_user_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session(&session).map_err(AuthError::ApiError)?;
        Ok(session)
    }

    pub async fn exchange_code_for_session(
        &self,
        auth_token: &str,
        refresh_token: Option<&str>,
    ) -> Result<TrailBaseSession, AuthError> {
        let claims = decode_jwt_claims(auth_token)
            .map_err(|e| AuthError::ApiError(format!("Failed to decode JWT: {}", e)))?;

        let now = Self::current_timestamp();
        let expires_at = now.saturating_add(3600);

        let session = TrailBaseSession {
            auth_token: auth_token.to_string(),
            refresh_token: refresh_token.unwrap_or("").to_string(),
            email: claims.email.clone().unwrap_or_default(),
            auth_user_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session(&session).map_err(AuthError::ApiError)?;
        Ok(session)
    }

    pub fn get_session_from_cookies() -> Result<TrailBaseSession, String> {
        let window = web_sys::window().ok_or("Window not available")?;
        let document = window.document().ok_or("Document not available")?;

        let cookie_value =
            js_sys::Reflect::get(&document, &wasm_bindgen::JsValue::from_str("cookie"))
                .map_err(|e| format!("Failed to get cookies: {:?}", e))?;

        let cookies = cookie_value.as_string().unwrap_or_default();

        let cookie_map: HashMap<&str, &str> = cookies
            .split(';')
            .filter_map(|cookie: &str| {
                let cookie = cookie.trim();
                let (key, value) = cookie.split_once('=')?;

                Some((key, value))
            })
            .collect();

        let auth_token = cookie_map
            .get("auth_token")
            .map(|s: &&str| s.to_string())
            .unwrap_or_default();
        let refresh_token = cookie_map
            .get("refresh_token")
            .map(|s: &&str| s.to_string())
            .unwrap_or_default();

        if auth_token.is_empty() {
            return Err("No auth_token found in cookies".to_string());
        }

        let claims = decode_jwt_claims(&auth_token)?;

        let now = Self::current_timestamp();
        let expires_at = now.saturating_add(3600);

        let session = TrailBaseSession {
            auth_token,
            refresh_token,
            email: claims.email.clone().unwrap_or_default(),
            auth_user_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session(&session).map_err(|e| format!("Failed to set session: {}", e))?;
        Ok(session)
    }

    pub fn parse_tokens_from_url(url_fragment: &str) -> Result<TrailBaseSession, String> {
        let fragment = url_fragment.strip_prefix('#').unwrap_or(url_fragment);

        let params: HashMap<&str, &str> = fragment
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.split('=');
                let key = parts.next()?;
                let value = parts.next()?;
                Some((key, value))
            })
            .collect();

        let auth_token = urlencoding_decode(params.get("auth_token").copied().unwrap_or_default());
        let refresh_token =
            urlencoding_decode(params.get("refresh_token").copied().unwrap_or_default());

        if auth_token.is_empty() {
            return Err("No auth_token found in URL fragment".to_string());
        }

        let claims = decode_jwt_claims(&auth_token)?;

        let now = Self::current_timestamp();
        let expires_at = now.saturating_add(3600);

        let session = TrailBaseSession {
            auth_token,
            refresh_token,
            email: claims.email.clone().unwrap_or_default(),
            auth_user_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session(&session).map_err(|e| format!("Failed to set session: {}", e))?;
        Ok(session)
    }

    pub async fn refresh_session(
        &self,
        refresh_token: &str,
    ) -> Result<TrailBaseSession, AuthError> {
        #[derive(Serialize)]
        struct RefreshRequest<'a> {
            refresh_token: &'a str,
        }

        let response = self
            .fetch(
                "/api/auth/v1/refresh",
                Method::POST,
                Some(&RefreshRequest { refresh_token }),
                None,
            )
            .await?;

        if !response.ok() {
            return Err(AuthError::SessionExpired);
        }

        let token_response: AuthTokenResponse = Self::json(response).await?;
        let claims = decode_jwt_claims(&token_response.auth_token)
            .map_err(|e| AuthError::ApiError(format!("Failed to decode JWT: {}", e)))?;

        let now = Self::current_timestamp();
        let expires_at = now.saturating_add(3600);

        let session = TrailBaseSession {
            auth_token: token_response.auth_token,
            refresh_token: token_response
                .refresh_token
                .unwrap_or_else(|| refresh_token.to_string()),
            email: claims.email.clone().unwrap_or_default(),
            auth_user_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session(&session).map_err(AuthError::ApiError)?;
        Ok(session)
    }

    pub async fn request_with_auth<T: Serialize>(
        &self,
        path: &str,
        method: Method,
        body: Option<&T>,
    ) -> Result<Response, AuthError> {
        use crate::repository::session::get_session;

        let session = get_session().ok_or(AuthError::SessionExpired)?;

        let session = if Self::should_refresh(session.expires_at) {
            if session.refresh_token.is_empty() {
                return Err(AuthError::SessionExpired);
            }
            self.refresh_session(&session.refresh_token).await?
        } else {
            session
        };

        let mut headers = HashMap::new();
        let auth_header = format!("Bearer {}", session.auth_token);
        headers.insert("Authorization".to_string(), auth_header);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = self
            .fetch(path, method.clone(), body, Some(headers))
            .await?;

        if response.status() == 401 {
            let session = get_session().ok_or(AuthError::SessionExpired)?;
            if session.refresh_token.is_empty() {
                return Err(AuthError::SessionExpired);
            }
            let refreshed = self.refresh_session(&session.refresh_token).await?;

            let mut headers = HashMap::new();
            let auth_header = format!("Bearer {}", refreshed.auth_token);
            headers.insert("Authorization".to_string(), auth_header);
            headers.insert("Content-Type".to_string(), "application/json".to_string());

            self.fetch(path, method, body, Some(headers)).await
        } else {
            Ok(response)
        }
    }

    pub async fn logout(&self) -> Result<(), String> {
        use crate::repository::session::{clear_session, get_session};

        if let Some(session) = get_session() {
            let auth_header = format!("Bearer {}", session.auth_token);
            let headers = HashMap::from([
                ("Authorization".to_string(), auth_header),
                ("Content-Type".to_string(), "application/json".to_string()),
            ]);
            let _ = self
                .fetch::<()>("/api/auth/v1/logout", Method::POST, None, Some(headers))
                .await;
        }
        clear_session();
        Ok(())
    }

    pub async fn delete_account(&self) -> Result<(), String> {
        use crate::repository::session::get_session;

        let session = get_session().ok_or("Not authenticated")?;

        let api = self.records("user");

        if let Some(record_id) = session.record_id {
            api.delete(&record_id.to_string())
                .await
                .map_err(|e| e.to_string())?;
        } else {
            let records: Vec<serde_json::Value> = api
                .list_filtered("email", &session.email)
                .await
                .map_err(|e| e.to_string())?;

            if let Some(record) = records.first()
                && let Some(id) = record.get("id").and_then(|v| v.as_i64())
            {
                api.delete(&id.to_string())
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }

        self.logout().await
    }

    pub fn records(&self, table_name: &str) -> RecordApi {
        RecordApi {
            client: self.clone(),
            table_name: table_name.to_string(),
        }
    }
}

impl Default for TrailBaseClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct RecordApi {
    client: TrailBaseClient,
    table_name: String,
}

impl RecordApi {
    pub async fn list<T: DeserializeOwned>(&self) -> Result<Vec<T>, AuthError> {
        let path = format!("/api/records/v1/{}", self.table_name);
        let response = self
            .client
            .request_with_auth(&path, Method::GET, None::<&()>)
            .await?;

        #[derive(Deserialize)]
        struct ListResponse<T> {
            records: Vec<T>,
        }

        let list: ListResponse<T> = TrailBaseClient::json(response).await?;
        Ok(list.records)
    }

    pub async fn list_filtered<T: DeserializeOwned>(
        &self,
        column: &str,
        value: &str,
    ) -> Result<Vec<T>, AuthError> {
        let path = format!(
            "/api/records/v1/{}?filter[{}][$eq]={}",
            self.table_name,
            urlencoding::encode(column),
            urlencoding::encode(value)
        );
        let response = self
            .client
            .request_with_auth(&path, Method::GET, None::<&()>)
            .await?;

        #[derive(Deserialize)]
        struct ListResponse<T> {
            records: Vec<T>,
        }

        let list: ListResponse<T> = TrailBaseClient::json(response).await?;
        Ok(list.records)
    }

    pub async fn read<T: DeserializeOwned>(&self, id: &str) -> Result<T, AuthError> {
        let path = format!("/api/records/v1/{}/{}", self.table_name, id);
        let response = self
            .client
            .request_with_auth(&path, Method::GET, None::<&()>)
            .await?;
        TrailBaseClient::json(response).await
    }

    pub async fn create<T: Serialize + std::fmt::Debug>(
        &self,
        record: &T,
    ) -> Result<String, AuthError> {
        let path = format!("/api/records/v1/{}", self.table_name);
        let response = self
            .client
            .request_with_auth(&path, Method::POST, Some(record))
            .await?;

        if !response.ok() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AuthError::ApiError(format!(
                "Failed to create record: {}",
                error_text
            )));
        }

        #[derive(Deserialize)]
        struct CreateResponse {
            ids: Vec<String>,
        }

        let create_response: CreateResponse = TrailBaseClient::json(response).await?;
        create_response
            .ids
            .first()
            .cloned()
            .ok_or_else(|| AuthError::ApiError("No ID returned".to_string()))
    }

    pub async fn update<T: Serialize>(&self, id: &str, record: &T) -> Result<(), AuthError> {
        let path = format!("/api/records/v1/{}/{}", self.table_name, id);
        let response = self
            .client
            .request_with_auth(&path, Method::PATCH, Some(record))
            .await?;

        if !response.ok() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AuthError::ApiError(format!(
                "Failed to update record: {}",
                error_text
            )));
        }

        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), AuthError> {
        let path = format!("/api/records/v1/{}/{}", self.table_name, id);
        let response = self
            .client
            .request_with_auth::<()>(&path, Method::DELETE, None)
            .await?;

        if !response.ok() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AuthError::ApiError(format!(
                "Failed to delete record: {}",
                error_text
            )));
        }

        Ok(())
    }
}

fn urlencoding_decode(s: &str) -> String {
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
