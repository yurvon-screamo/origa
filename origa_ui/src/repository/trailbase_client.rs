use crate::repository::session::{TrailBaseSession, set_session_async};
use crate::repository::trailbase_auth::{decode_jwt_claims, urlencoding_decode};
use crate::repository::trailbase_records::RecordApi;
use crate::repository::trailbase_session::current_timestamp;

use gloo_net::http::{Method, Request, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::OnceLock;
use thiserror::Error;

pub use crate::repository::trailbase_auth::OAuthProvider;
pub type TrailBaseRecordApi = RecordApi<TrailBaseClient>;

// HTTP transport layer: fetch, json, login, OAuth, token exchange.
// Session lifecycle (refresh, logout, password change): see trailbase_session.rs

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
pub(crate) struct AuthTokenResponse {
    pub(crate) auth_token: String,
    pub(crate) refresh_token: Option<String>,
}

/// Compile-time TrailBase API base URL.
///
/// Exposed as `pub(crate)` so the OAuth `redirect_uri` builder can reuse the
/// canonical backend host (`https://app.origa.uwuwu.net` in production) instead
/// of `ORIGA_PUBLIC_BASE_URL`, which has been empty since commit `eeee03ad`
/// (mobile OIDC redirect refactor) and produced a relative `redirect_uri`
/// that TrailBase could not redirect to.
pub(crate) fn trailbase_url() -> &'static str {
    static TRAILBASE_URL: OnceLock<&str> = OnceLock::new();
    TRAILBASE_URL.get_or_init(|| env!("TRAILBASE_URL"))
}

#[derive(Clone, Debug)]
pub struct TrailBaseClient {
    base_url: String,
}

impl TrailBaseClient {
    pub fn new() -> Self {
        Self {
            base_url: trailbase_url().to_string(),
        }
    }

    pub(crate) async fn fetch<T: Serialize>(
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
            },
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

    pub(crate) async fn json<T: DeserializeOwned>(response: Response) -> Result<T, AuthError> {
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

        let now = current_timestamp();
        let expires_at = claims.expires_at(now.saturating_add(3600));

        let session = TrailBaseSession {
            auth_token: token_response.auth_token,
            refresh_token: token_response.refresh_token.unwrap_or_default(),
            email: claims.email.clone().unwrap_or_default(),
            trailbase_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session_async(&session)
            .await
            .map_err(AuthError::ApiError)?;
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

        let now = current_timestamp();
        let expires_at = claims.expires_at(now.saturating_add(3600));

        let session = TrailBaseSession {
            auth_token: token_response.auth_token,
            refresh_token: token_response.refresh_token.unwrap_or_default(),
            email: claims.email.clone().unwrap_or_default(),
            trailbase_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session_async(&session)
            .await
            .map_err(AuthError::ApiError)?;
        Ok(session)
    }

    /// Parses tokens from a URL fragment. Does NOT persist the session — the
    /// caller is responsible for calling `set_session_async`.
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

        let now = current_timestamp();
        let expires_at = claims.expires_at(now.saturating_add(3600));

        let session = TrailBaseSession {
            auth_token,
            refresh_token,
            email: claims.email.clone().unwrap_or_default(),
            trailbase_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        Ok(session)
    }

    pub fn records(&self, table_name: &str) -> TrailBaseRecordApi {
        RecordApi::new(self.clone(), table_name.to_string())
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
