use super::session::{SupabaseSession, SupabaseUser};
use crate::repository::session::{clear_session, get_session, set_session};
use base64::{self, Engine};
use reqwest::{
    Client, Method, RequestBuilder, StatusCode,
    header::{HeaderMap, HeaderValue},
};
use serde_json::Value;

const SUPABASE_URL: &str = "https://evttbadnaklzjnxhwqad.supabase.co";
const SUPABASE_PUBLISHABLE_KEY: &str = "sb_publishable_SScoXTXJy1tQFVJr2_9mXQ_77Q3aetA";
const REFRESH_THRESHOLD_SECONDS: u64 = 300;
const OAUTH_REDIRECT_URI: &str = "origa://auth/callback";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthProvider {
    Google,
    Yandex,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::Yandex => "keycloak",
        }
    }
}

#[derive(Clone)]
pub struct SupabaseClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug)]
pub enum AuthError {
    SessionExpired,
    NetworkError(String),
    ApiError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::SessionExpired => write!(f, "Session expired, please login again"),
            AuthError::NetworkError(e) => write!(f, "Network error: {}", e),
            AuthError::ApiError(e) => write!(f, "API error: {}", e),
        }
    }
}

impl std::error::Error for AuthError {}

impl SupabaseClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "apikey",
            HeaderValue::from_str(SUPABASE_PUBLISHABLE_KEY).unwrap(),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: SUPABASE_URL.to_string(),
            api_key: SUPABASE_PUBLISHABLE_KEY.to_string(),
        }
    }

    pub fn request(&self, method: Method, url: &str, auth_token: Option<&str>) -> RequestBuilder {
        let target_url = format!("{}{}", self.base_url, url);
        let mut request = self
            .client
            .request(method, target_url)
            .header("Content-Type", "application/json")
            .header("apikey", &self.api_key);

        if let Some(auth_token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", auth_token));
        }

        request
    }

    fn current_timestamp() -> u64 {
        chrono::Utc::now().timestamp() as u64
    }

    fn should_refresh(expires_at: u64) -> bool {
        let now = Self::current_timestamp();
        now.saturating_add(REFRESH_THRESHOLD_SECONDS) >= expires_at
    }

    pub async fn refresh_session(&self, refresh_token: &str) -> Result<SupabaseSession, AuthError> {
        let res = self
            .request(
                Method::POST,
                "/auth/v1/token?grant_type=refresh_token",
                None,
            )
            .json(&serde_json::json!({
                "refresh_token": refresh_token,
            }))
            .send()
            .await
            .map_err(|e| AuthError::NetworkError(e.to_string()))?;

        if !res.status().is_success() {
            return Err(AuthError::SessionExpired);
        }

        let json: Value = res
            .json()
            .await
            .map_err(|e| AuthError::ApiError(format!("Failed to parse response: {}", e)))?;

        let access_token = json
            .get("access_token")
            .and_then(|t| t.as_str())
            .unwrap_or_default()
            .to_string();
        let new_refresh_token = json
            .get("refresh_token")
            .and_then(|t| t.as_str())
            .unwrap_or_default()
            .to_string();
        let expires_in = json
            .get("expires_in")
            .and_then(|t| t.as_i64())
            .unwrap_or(3600) as u64;

        let (auth_user_id, user_email) = Self::decode_jwt_payload(&access_token)
            .unwrap_or_else(|| (String::new(), String::new()));

        let now = Self::current_timestamp();
        let expires_at = now.saturating_add(expires_in);

        let session = SupabaseSession {
            access_token,
            refresh_token: new_refresh_token,
            auth_user_id,
            email: user_email,
            expires_at,
        };

        set_session(&session).map_err(AuthError::ApiError)?;
        Ok(session)
    }

    pub async fn request_with_auth_refresh(
        &self,
        method: Method,
        url: &str,
        body: Option<&Value>,
        extra_headers: Option<&[(&str, &str)]>,
    ) -> Result<reqwest::Response, AuthError> {
        let session = get_session().ok_or(AuthError::SessionExpired)?;

        let session = if Self::should_refresh(session.expires_at) {
            self.refresh_session(&session.refresh_token).await?
        } else {
            session
        };

        let res = self
            .build_request(
                method.clone(),
                url,
                &session.access_token,
                body,
                extra_headers,
            )
            .send()
            .await
            .map_err(|e| AuthError::NetworkError(e.to_string()))?;

        if res.status() == StatusCode::UNAUTHORIZED {
            let session = get_session().ok_or(AuthError::SessionExpired)?;
            let refreshed = self.refresh_session(&session.refresh_token).await?;

            self.build_request(method, url, &refreshed.access_token, body, extra_headers)
                .send()
                .await
                .map_err(|e| AuthError::NetworkError(e.to_string()))
        } else {
            Ok(res)
        }
    }

    fn build_request(
        &self,
        method: Method,
        url: &str,
        auth_token: &str,
        body: Option<&Value>,
        extra_headers: Option<&[(&str, &str)]>,
    ) -> RequestBuilder {
        let mut request = self.request(method, url, Some(auth_token));

        if let Some(json_body) = body {
            request = request.json(json_body);
        }

        if let Some(headers) = extra_headers {
            for (key, value) in headers {
                request = request.header(*key, *value);
            }
        }

        request
    }

    fn decode_jwt_payload(token: &str) -> Option<(String, String)> {
        let payload_base64 = token.split(".").nth(1)?;
        let output_size = base64::decoded_len_estimate(payload_base64.len());
        let mut payload_buffer = Vec::<u8>::with_capacity(output_size);
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode_vec(payload_base64, &mut payload_buffer)
            .ok()?;
        let payload_json: Value = serde_json::from_slice(&payload_buffer[..]).ok()?;
        let uuid = payload_json.get("sub")?.as_str()?.to_owned();
        let email = payload_json.get("email")?.as_str()?.to_owned();
        Some((uuid, email))
    }

    fn parse_supabase_error(error_text: &str) -> String {
        if let Ok(json) = serde_json::from_str::<Value>(error_text) {
            json.get("msg")
                .and_then(|m| m.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| error_text.to_string())
        } else {
            error_text.to_string()
        }
    }

    pub async fn register(&self, email: &str, password: &str) -> Result<SupabaseUser, String> {
        let res = self
            .request(Method::POST, "/auth/v1/signup", None)
            .json(&serde_json::json!({
                "email": email,
                "password": password,
            }))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if res.status().is_success() {
            let json: Value = res
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            let user = SupabaseUser {
                id: json
                    .get("id")
                    .and_then(|id| id.as_str())
                    .unwrap_or_default()
                    .to_string(),
                email: json
                    .get("email")
                    .and_then(|e| e.as_str())
                    .unwrap_or(email)
                    .to_string(),
            };
            Ok(user)
        } else {
            let error_text = res.text().await.unwrap_or_default();
            Err(Self::parse_supabase_error(&error_text))
        }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<SupabaseSession, String> {
        let res = self
            .request(Method::POST, "/auth/v1/token?grant_type=password", None)
            .json(&serde_json::json!({
                "email": email,
                "password": password,
            }))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if res.status().is_success() {
            let json: Value = res
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let access_token = json
                .get("access_token")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string();
            let refresh_token = json
                .get("refresh_token")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string();
            let expires_in = json
                .get("expires_in")
                .and_then(|t| t.as_i64())
                .unwrap_or(3600) as u64;

            let (user_id, user_email) = Self::decode_jwt_payload(&access_token)
                .unwrap_or_else(|| (String::new(), email.to_string()));

            let now = Self::current_timestamp();
            let expires_at = now.saturating_add(expires_in);

            let session = SupabaseSession {
                access_token,
                refresh_token,
                auth_user_id: user_id,
                email: user_email,
                expires_at,
            };

            set_session(&session).map_err(|e| format!("Failed to set session: {}", e))?;
            Ok(session)
        } else {
            let error_text = res.text().await.unwrap_or_default();
            Err(Self::parse_supabase_error(&error_text))
        }
    }

    pub fn get_oauth_url(provider: &str) -> String {
        format!(
            "{}/auth/v1/authorize?provider={}&redirect_to={}&scopes=email%20profile",
            SUPABASE_URL, provider, OAUTH_REDIRECT_URI
        )
    }

    pub fn get_oauth_url_with_redirect(provider: &str, redirect_uri: &str) -> String {
        let encoded = urlencoding::encode(redirect_uri);
        format!(
            "{}/auth/v1/authorize?provider={}&redirect_to={}&scopes=email%20profile",
            SUPABASE_URL, provider, encoded
        )
    }

    pub async fn exchange_code_for_session(&self, code: &str) -> Result<SupabaseSession, String> {
        let res = self
            .request(Method::POST, "/auth/v1/token?grant_type=pkce", None)
            .json(&serde_json::json!({
                "auth_code": code,
            }))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if res.status().is_success() {
            let json: Value = res
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let access_token = json
                .get("access_token")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string();
            let refresh_token = json
                .get("refresh_token")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string();
            let expires_in = json
                .get("expires_in")
                .and_then(|t| t.as_i64())
                .unwrap_or(3600) as u64;

            let (user_id, user_email) = Self::decode_jwt_payload(&access_token)
                .unwrap_or_else(|| (String::new(), String::new()));

            let now = Self::current_timestamp();
            let expires_at = now.saturating_add(expires_in);

            let session = SupabaseSession {
                access_token,
                refresh_token,
                auth_user_id: user_id,
                email: user_email,
                expires_at,
            };

            set_session(&session).map_err(|e| format!("Failed to set session: {}", e))?;
            Ok(session)
        } else {
            let error_text = res.text().await.unwrap_or_default();
            Err(Self::parse_supabase_error(&error_text))
        }
    }

    pub fn parse_tokens_from_url(url_fragment: &str) -> Result<SupabaseSession, String> {
        let fragment = url_fragment.strip_prefix('#').unwrap_or(url_fragment);

        let params: std::collections::HashMap<&str, &str> = fragment
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.split('=');
                let key = parts.next()?;
                let value = parts.next()?;
                Some((key, value))
            })
            .collect();

        let access_token =
            urlencoding_decode(params.get("access_token").copied().unwrap_or_default());
        let refresh_token =
            urlencoding_decode(params.get("refresh_token").copied().unwrap_or_default());
        let expires_in: u64 = params
            .get("expires_in")
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600);

        if access_token.is_empty() {
            return Err("No access_token found in URL fragment".to_string());
        }

        let (user_id, user_email) = Self::decode_jwt_payload(&access_token)
            .unwrap_or_else(|| (String::new(), String::new()));

        let now = Self::current_timestamp();
        let expires_at = now.saturating_add(expires_in);

        let session = SupabaseSession {
            access_token,
            refresh_token,
            auth_user_id: user_id,
            email: user_email,
            expires_at,
        };

        set_session(&session).map_err(|e| format!("Failed to set session: {}", e))?;
        Ok(session)
    }

    pub async fn resend_confirmation_email(&self, email: &str) -> Result<(), String> {
        let res = self
            .request(Method::POST, "/auth/v1/resend", None)
            .json(&serde_json::json!({
                "type": "signup",
                "email": email,
            }))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if res.status().is_success() {
            Ok(())
        } else {
            let error_text = res.text().await.unwrap_or_default();
            Err(Self::parse_supabase_error(&error_text))
        }
    }

    pub async fn logout(&self) -> Result<(), String> {
        if let Some(session) = get_session() {
            let _ = self
                .request(Method::POST, "/auth/v1/logout", Some(&session.access_token))
                .send()
                .await;
        }
        clear_session();
        Ok(())
    }

    pub async fn delete_account(&self) -> Result<(), String> {
        let session = get_session().ok_or("Not authenticated")?;

        let res = self
            .request(
                Method::DELETE,
                &format!("/rest/v1/user?auth_user_id=eq.{}", session.auth_user_id),
                Some(&session.access_token),
            )
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(format!("Failed to delete profile: {}", error_text));
        }

        clear_session();
        Ok(())
    }
}

impl Default for SupabaseClient {
    fn default() -> Self {
        Self::new()
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
