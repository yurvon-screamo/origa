use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TrailBaseSession {
    pub auth_token: String,
    pub refresh_token: String,
    pub email: String,
    pub auth_user_id: String,
    pub record_id: Option<i64>,
    pub expires_at: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TrailBaseUser {
    pub sub: String,
    pub email: String,
}

const SESSION_KEY: &str = "trailbase_session";

pub fn get_session() -> Option<TrailBaseSession> {
    LocalStorage::get(SESSION_KEY).ok()
}

pub fn set_session(session: &TrailBaseSession) -> Result<(), String> {
    LocalStorage::set(SESSION_KEY, session).map_err(|e| format!("Failed to set session: {}", e))
}

pub fn clear_session() {
    LocalStorage::delete(SESSION_KEY);
}
