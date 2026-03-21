use crate::repository::session::{TrailBaseSession, set_session};
use crate::repository::trailbase_auth::{decode_jwt_claims, urlencoding_decode};
use crate::repository::trailbase_records::RecordApi;

use gloo_net::http::{Method, Request, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::OnceLock;
use thiserror::Error;

pub use crate::repository::trailbase_auth::OAuthProvider;
pub type TrailBaseRecordApi = RecordApi<TrailBaseClient>;

const REFRESH_THRESHOLD_SECONDS: u64 = 300;

fn trailbase_url() -> &'static str {
    static TRAILBASE_URL: OnceLock<&str> = OnceLock::new();
    TRAILBASE_URL.get_or_init(|| option_env!("TRAILBASE_URL").unwrap_or("https://origa.uwuwu.net"))
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
}

impl TrailBaseClient {
    pub fn new() -> Self {
        Self {
            base_url: trailbase_url().to_string(),
        }
    }

    #[allow(dead_code)]
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

    pub fn get_oauth_url(
        &self,
        provider: &str,
        redirect_uri: &str,
        pkce_challenge: &str,
    ) -> String {
        let encoded_redirect = urlencoding::encode(redirect_uri);
        let encoded_challenge = urlencoding::encode(pkce_challenge);

        let scope = if provider == "oidc0" {
            "&scope=login:email%20login:info"
        } else {
            ""
        };

        format!(
            "{}/api/auth/v1/oauth/{}/login?redirect_uri={}&response_type=code&pkce_code_challenge={}{}",
            self.base_url, provider, encoded_redirect, encoded_challenge, scope
        )
    }

    pub async fn login_with_email_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<TrailBaseSession, AuthError> {
        #[derive(Serialize)]
        struct LoginRequest<'a> {
            email: &'a str,
            password: &'a str,
        }

        let response = self
            .fetch(
                "/api/auth/v1/login",
                Method::POST,
                Some(&LoginRequest { email, password }),
                None,
            )
            .await?;

        if !response.ok() {
            return Err(AuthError::ApiError(format!(
                "Login failed: {}",
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
            trailbase_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session(&session).map_err(AuthError::ApiError)?;
        Ok(session)
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
            trailbase_id: claims.sub.clone(),
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
            trailbase_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session(&session).map_err(AuthError::ApiError)?;
        Ok(session)
    }

    #[allow(dead_code)]
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
            trailbase_id: claims.sub.clone(),
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
            trailbase_id: claims.sub.clone(),
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
            trailbase_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session(&session).map_err(AuthError::ApiError)?;
        Ok(session)
    }

    async fn _request_with_auth_impl<T: Serialize>(
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

    #[allow(dead_code)]
    pub async fn request_with_auth<T: Serialize>(
        &self,
        path: &str,
        method: Method,
        body: Option<&T>,
    ) -> Result<Response, AuthError> {
        self._request_with_auth_impl(path, method, body).await
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

    pub fn records(&self, table_name: &str) -> TrailBaseRecordApi {
        RecordApi::new(self.clone(), table_name.to_string())
    }

    #[allow(dead_code)]
    pub fn generate_pkce_verifier() -> String {
        super::trailbase_auth::generate_pkce_verifier()
    }

    #[allow(dead_code)]
    pub fn generate_pkce_challenge(verifier: &str) -> String {
        super::trailbase_auth::generate_pkce_challenge(verifier)
    }
}

impl Default for TrailBaseClient {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(async_fn_in_trait)]
pub trait AuthRequestClient: Clone + Send + Sync {
    async fn request_with_auth<T: Serialize>(
        &self,
        path: &str,
        method: Method,
        body: Option<&T>,
    ) -> Result<Response, AuthError>;
}

impl AuthRequestClient for TrailBaseClient {
    async fn request_with_auth<T: Serialize>(
        &self,
        path: &str,
        method: Method,
        body: Option<&T>,
    ) -> Result<Response, AuthError> {
        self._request_with_auth_impl(path, method, body).await
    }
}
