use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SupabaseSession {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub email: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SupabaseUser {
    pub id: String,
    pub email: String,
}

const SESSION_KEY: &str = "supabase_session";

pub fn get_session() -> Option<SupabaseSession> {
    LocalStorage::get(SESSION_KEY).ok()
}

pub fn set_session(session: &SupabaseSession) -> Result<(), String> {
    LocalStorage::set(SESSION_KEY, session).map_err(|e| format!("Failed to set session: {}", e))
}

pub fn clear_session() {
    LocalStorage::delete(SESSION_KEY);
}
