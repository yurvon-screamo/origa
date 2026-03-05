use super::trailbase_client::{AuthError, TrailBaseClient};
use crate::repository::session::{TrailBaseSession, get_session, set_session};
use chrono::{DateTime, Utc};
use origa::application::user_repository::UserRepository;
use origa::domain::{JlptProgress, KnowledgeSet, NativeLanguage, OrigaError, User};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use ulid::Ulid;

#[derive(Clone)]
pub struct TrailBaseUserRepository {
    client: TrailBaseClient,
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

impl TrailBaseUserRepository {
    pub fn new() -> Self {
        Self {
            client: TrailBaseClient::new(),
            table_name: "user".to_string(),
            user_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn find_current(&self) -> Result<Option<(User, i64)>, OrigaError> {
        let session = get_session().ok_or_else(|| OrigaError::RepositoryError {
            reason: "Not authenticated".to_string(),
        })?;

        if session.email.is_empty() {
            return Err(OrigaError::RepositoryError {
                reason: "Email not found in session. Please re-login.".to_string(),
            });
        }

        let api = self.client.records(&self.table_name);

        let records: Vec<UserRow> = api
            .list_filtered("email", &session.email)
            .await
            .map_err(map_auth_error)?;

        if let Some(row) = records.into_iter().next() {
            let record_id = row.id.ok_or_else(|| OrigaError::RepositoryError {
                reason: "Record ID missing from database row".to_string(),
            })?;
            let user = row.to_user();

            if let Ok(mut cache) = self.user_cache.write() {
                cache.insert(session.email.clone(), user.clone());
            }

            return Ok(Some((user, record_id)));
        }

        Ok(None)
    }
}

impl Default for TrailBaseUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct UserRow {
    #[serde(default)]
    id: Option<i64>,
    auth_user_id: String,
    username: String,
    email: String,
    native_language: i32,
    #[serde(default)]
    jlpt_progress: Option<JlptProgress>,
    current_japanese_level: Option<i32>,
    duolingo_jwt_token: Option<String>,
    telegram_user_id: Option<i64>,
    reminders_enabled: i32,
    knowledge_set: KnowledgeSet,
    updated_at: DateTime<Utc>,
}

impl UserRow {
    fn to_user(&self) -> User {
        let ulid = uuid_to_ulid(&self.auth_user_id);

        let jlpt_progress = self.jlpt_progress.clone().unwrap_or_default();

        User::from_row(
            ulid,
            self.email.clone(),
            self.username.clone(),
            jlpt_progress,
            NativeLanguage::from(self.native_language),
            self.duolingo_jwt_token.clone(),
            self.telegram_user_id.map(|id| id as u64),
            self.reminders_enabled != 0,
            self.knowledge_set.clone(),
            self.updated_at,
        )
    }
}

fn uuid_to_ulid(uuid_str: &str) -> Ulid {
    let uuid_bytes = uuid_str
        .replace('-', "")
        .as_bytes()
        .chunks(2)
        .filter_map(|chunk| {
            let hex = std::str::from_utf8(chunk).ok()?;
            u8::from_str_radix(hex, 16).ok()
        })
        .collect::<Vec<_>>();

    let mut bytes = [0u8; 16];
    if uuid_bytes.len() == 16 {
        bytes.copy_from_slice(&uuid_bytes);
    }

    Ulid::from_bytes(bytes)
}

fn user_to_json(user: &User, auth_user_id: &str) -> serde_json::Value {
    let jlpt_progress_json = serde_json::to_string(user.jlpt_progress()).unwrap_or_else(|_| "null".to_string());
    
    let mut map = serde_json::Map::new();
    map.insert("auth_user_id".to_string(), serde_json::json!(auth_user_id));
    map.insert("username".to_string(), serde_json::json!(user.username()));
    map.insert("email".to_string(), serde_json::json!(user.email()));
    map.insert("native_language".to_string(), serde_json::json!(i32::from(user.native_language().clone())));
    map.insert("current_japanese_level".to_string(), serde_json::json!(i32::from(user.current_japanese_level())));
    map.insert("jlpt_progress".to_string(), serde_json::json!(jlpt_progress_json));
    map.insert("reminders_enabled".to_string(), serde_json::json!(if user.reminders_enabled() { 1 } else { 0 }));
    map.insert("knowledge_set".to_string(), serde_json::json!(user.knowledge_set()));
    
    if let Some(token) = user.duolingo_jwt_token() {
        map.insert("duolingo_jwt_token".to_string(), serde_json::json!(token));
    }
    if let Some(id) = user.telegram_user_id().copied() {
        map.insert("telegram_user_id".to_string(), serde_json::json!(id as i64));
    }
    
    serde_json::Value::Object(map)
}

impl UserRepository for TrailBaseUserRepository {
    async fn find_by_id(&self, _user_id: Ulid) -> Result<Option<User>, OrigaError> {
        self.find_current()
            .await
            .map(|opt| opt.map(|(user, _)| user))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, OrigaError> {
        if let Ok(cache) = self.user_cache.read()
            && let Some(user) = cache.get(email)
        {
            return Ok(Some(user.clone()));
        }
        self.find_current()
            .await
            .map(|opt| opt.map(|(user, _)| user))
    }

    async fn find_by_telegram_id(&self, _telegram_id: &u64) -> Result<Option<User>, OrigaError> {
        self.find_current()
            .await
            .map(|opt| opt.map(|(user, _)| user))
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        let session = get_session().ok_or_else(|| OrigaError::RepositoryError {
            reason: "Not authenticated".to_string(),
        })?;

        if session.email.is_empty() {
            return Err(OrigaError::RepositoryError {
                reason: "Email not found in session. Please re-login.".to_string(),
            });
        }

        let api = self.client.records(&self.table_name);
        let body = user_to_json(user, &session.auth_user_id);

        web_sys::console::log_1(&format!("Saving user, body: {}", serde_json::to_string(&body).unwrap_or_default()).into());

        if let Some((_, record_id)) = self.find_current().await? {
            web_sys::console::log_1(&format!("Updating existing record: {}", record_id).into());
            api.update(&record_id.to_string(), &body)
                .await
                .map_err(map_auth_error)?;
        } else {
            web_sys::console::log_1(&"Creating new record".into());
            let created_id = api.create(&body).await.map_err(|e| {
                web_sys::console::log_1(&format!("Create error: {:?}", e).into());
                map_auth_error(e)
            })?;
            let record_id: i64 = created_id
                .parse()
                .map_err(|_| OrigaError::RepositoryError {
                    reason: "Invalid record ID returned from create".to_string(),
                })?;

            let updated_session = TrailBaseSession {
                record_id: Some(record_id),
                ..session.clone()
            };
            set_session(&updated_session).map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to update session: {}", e),
            })?;
        }

        if let Ok(mut cache) = self.user_cache.write() {
            cache.insert(user.email().to_string(), user.clone());
        }

        Ok(())
    }

    async fn delete(&self, _user_id: Ulid) -> Result<(), OrigaError> {
        let _session = get_session().ok_or_else(|| OrigaError::RepositoryError {
            reason: "Not authenticated".to_string(),
        })?;

        let api = self.client.records(&self.table_name);

        if let Some((_, record_id)) = self.find_current().await? {
            api.delete(&record_id.to_string())
                .await
                .map_err(map_auth_error)?;
        }

        if let Ok(mut cache) = self.user_cache.write() {
            cache.clear();
        }

        Ok(())
    }
}
