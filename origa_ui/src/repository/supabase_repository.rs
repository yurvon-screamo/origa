use super::client::{AuthError, SupabaseClient};
use crate::repository::session::get_session;
use origa::application::user_repository::UserRepository;
use origa::domain::{JapaneseLevel, KnowledgeSet, NativeLanguage, OrigaError, User};
use reqwest::Method;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use ulid::Ulid;

#[derive(Clone)]
pub struct SupabaseUserRepository {
    client: SupabaseClient,
    table_name: String,
    user_cache: Arc<RwLock<HashMap<String, User>>>,
}

fn map_auth_error(e: AuthError) -> OrigaError {
    match e {
        AuthError::SessionExpired => OrigaError::SessionExpired,
        AuthError::NetworkError(msg) => OrigaError::RepositoryError {
            reason: format!("Network error: {}", msg),
        },
        AuthError::ApiError(msg) => OrigaError::RepositoryError {
            reason: format!("API error: {}", msg),
        },
    }
}

impl SupabaseUserRepository {
    pub fn new() -> Self {
        Self {
            client: SupabaseClient::new(),
            table_name: "user".to_string(),
            user_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn find_current(&self) -> Result<Option<User>, OrigaError> {
        let session = get_session().ok_or_else(|| OrigaError::RepositoryError {
            reason: "Not authenticated".to_string(),
        })?;

        let res = self
            .client
            .request_with_auth_refresh(
                Method::GET,
                &format!(
                    "/rest/v1/{}?auth_user_id=eq.{}&select=*",
                    self.table_name, session.user_id
                ),
                None,
                None,
            )
            .await
            .map_err(map_auth_error)?;

        if res.status().is_success() {
            let rows: Vec<UserRow> = res.json().await.map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to parse response: {}", e),
            })?;

            if let Some(row) = rows.first() {
                let user = row.to_user();
                if let Ok(mut cache) = self.user_cache.write() {
                    cache.insert(session.email.clone(), user.clone());
                }
                return Ok(Some(user));
            }
        }

        Ok(None)
    }
}

impl Default for SupabaseUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Deserialize)]
struct UserRow {
    id: i64,
    auth_user_id: String,
    username: String,
    native_language: i32,
    current_japanese_level: i32,
    duolingo_jwt_token: Option<String>,
    telegram_user_id: Option<i64>,
    reminders_enabled: bool,
    knowledge_set: KnowledgeSet,
}

impl UserRow {
    fn to_user(&self) -> User {
        let ulid = supabase_id_to_ulid(self.id);

        User::from_row(
            ulid,
            self.auth_user_id.clone(),
            self.username.clone(),
            JapaneseLevel::from(self.current_japanese_level),
            NativeLanguage::from(self.native_language),
            self.duolingo_jwt_token.clone(),
            self.telegram_user_id.map(|id| id as u64),
            self.reminders_enabled,
            self.knowledge_set.clone(),
        )
    }
}

fn supabase_id_to_ulid(id: i64) -> Ulid {
    let mut bytes = [0u8; 16];
    let id_bytes = id.to_be_bytes();
    bytes[8..16].copy_from_slice(&id_bytes);
    Ulid::from_bytes(bytes)
}

fn user_to_json(user: &User, auth_user_id: &str) -> serde_json::Value {
    serde_json::json!({
        "auth_user_id": auth_user_id,
        "username": user.username(),
        "native_language": i32::from(user.native_language().clone()),
        "current_japanese_level": i32::from(*user.current_japanese_level()),
        "duolingo_jwt_token": user.duolingo_jwt_token(),
        "telegram_user_id": user.telegram_user_id().copied().map(|id| id as i64),
        "reminders_enabled": user.reminders_enabled(),
        "knowledge_set": user.knowledge_set(),
    })
}

impl UserRepository for SupabaseUserRepository {
    async fn list(&self) -> Result<Vec<User>, OrigaError> {
        let user = self.find_current().await?;
        Ok(user.map(|u| vec![u]).unwrap_or_default())
    }

    async fn find_by_id(&self, _user_id: Ulid) -> Result<Option<User>, OrigaError> {
        self.find_current().await
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, OrigaError> {
        if let Ok(cache) = self.user_cache.read()
            && let Some(user) = cache.get(email)
        {
            return Ok(Some(user.clone()));
        }
        self.find_current().await
    }

    async fn find_by_telegram_id(&self, _telegram_id: &u64) -> Result<Option<User>, OrigaError> {
        self.find_current().await
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        let session = get_session().ok_or_else(|| OrigaError::RepositoryError {
            reason: "Not authenticated".to_string(),
        })?;

        let existing = self.find_current().await?;
        let body = user_to_json(user, &session.user_id);

        if existing.is_some() {
            let res = self
                .client
                .request_with_auth_refresh(
                    Method::PATCH,
                    &format!(
                        "/rest/v1/{}?auth_user_id=eq.{}",
                        self.table_name, session.user_id
                    ),
                    Some(&body),
                    Some(&[("Prefer", "return=minimal")]),
                )
                .await
                .map_err(map_auth_error)?;

            if !res.status().is_success() {
                let error_text = res.text().await.unwrap_or_default();
                return Err(OrigaError::RepositoryError {
                    reason: format!("Failed to update user: {}", error_text),
                });
            }
        } else {
            let res = self
                .client
                .request_with_auth_refresh(
                    Method::POST,
                    &format!("/rest/v1/{}", self.table_name),
                    Some(&body),
                    Some(&[("Prefer", "return=minimal")]),
                )
                .await
                .map_err(map_auth_error)?;

            if !res.status().is_success() {
                let error_text = res.text().await.unwrap_or_default();
                return Err(OrigaError::RepositoryError {
                    reason: format!("Failed to create user: {}", error_text),
                });
            }
        }

        if let Ok(mut cache) = self.user_cache.write() {
            cache.insert(user.email().to_string(), user.clone());
        }

        Ok(())
    }

    async fn delete(&self, _user_id: Ulid) -> Result<(), OrigaError> {
        let session = get_session().ok_or_else(|| OrigaError::RepositoryError {
            reason: "Not authenticated".to_string(),
        })?;

        let res = self
            .client
            .request_with_auth_refresh(
                Method::DELETE,
                &format!(
                    "/rest/v1/{}?auth_user_id=eq.{}",
                    self.table_name, session.user_id
                ),
                None,
                None,
            )
            .await
            .map_err(map_auth_error)?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(OrigaError::RepositoryError {
                reason: format!("Failed to delete user: {}", error_text),
            });
        }

        if let Ok(mut cache) = self.user_cache.write() {
            cache.clear();
        }

        Ok(())
    }
}
