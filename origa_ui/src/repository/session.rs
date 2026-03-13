use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TrailBaseSession {
    pub auth_token: String,
    pub refresh_token: String,
    pub email: String,
    pub trailbase_id: String,
    pub record_id: Option<i64>,
    pub expires_at: u64,
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

const LAST_SYNC_KEY: &str = "origa_last_sync_time";

pub fn get_last_sync_time() -> u64 {
    LocalStorage::get(LAST_SYNC_KEY).unwrap_or(0)
}

pub fn set_last_sync_time(timestamp: u64) {
    let _ = LocalStorage::set(LAST_SYNC_KEY, timestamp);
}

pub fn clear_last_sync_time() {
    LocalStorage::delete(LAST_SYNC_KEY);
}
