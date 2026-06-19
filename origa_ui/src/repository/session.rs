//! TrailBase session storage with three-tier fallback strategy:
//!
//! 1. **In-memory cache** (`OnceLock<Arc<RwLock<…>>>`) — hot path, sync reads.
//! 2. **Tauri plugin store** (`auth.json` on native FS) — persists across
//!    process kills on Android; only available inside a Tauri WebView.
//! 3. **WebView `localStorage`** — web-build primary; Tauri legacy/migration.
//!
//! On Tauri, the async variants (`*_async`) write to the store with explicit
//! `Store::save()` fsync via IPC commands, then update the cache + localStorage.
//! The sync variants (`get_session`/`set_session`/`clear_session`) read/write
//! the cache + localStorage only, and serve as the web-build fallback.

use gloo_storage::{LocalStorage, Storage};
use js_sys::{Object, Reflect};
use leptos::wasm_bindgen::JsValue;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock, RwLock};
use tracing::{debug, warn};

use crate::core::tauri;

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
const PKCE_VERIFIER_KEY: &str = "pkce_verifier";

// ── Global session cache (AD-2: single source of truth) ──────────────
//
// `std::sync::RwLock` is safe here because (a) origa_ui is WASM and thus
// single-threaded, and (b) we never hold the guard across an await point.
// A `tokio::sync::RwLock` would require adding tokio as a dependency to
// origa_ui, which is undesirable for a CSR-only WASM crate.

static SESSION_CACHE: OnceLock<Arc<RwLock<Option<TrailBaseSession>>>> = OnceLock::new();

fn session_cache() -> &'static Arc<RwLock<Option<TrailBaseSession>>> {
    SESSION_CACHE.get_or_init(|| Arc::new(RwLock::new(None)))
}

fn cache_read() -> Option<TrailBaseSession> {
    session_cache().read().ok().and_then(|guard| guard.clone())
}

fn cache_write(session: Option<TrailBaseSession>) {
    if let Ok(mut guard) = session_cache().write() {
        *guard = session;
    }
}

// ── Sync API (cache + localStorage) ──────────────────────────────────

pub fn get_session() -> Option<TrailBaseSession> {
    if let Some(cached) = cache_read() {
        return Some(cached);
    }
    // On Tauri, the cache is the only reliable sync source. localStorage is
    // unreliable under process kills (Chromium DOMStorage bug 479767) and
    // returning a stale entry from it could cause authenticated requests to
    // use an invalid session. The cache is populated by get_session_async()
    // during check_session at app start.
    //
    // On web, localStorage IS the primary persistent store.
    if tauri::is_tauri() {
        return None;
    }

    let session: Option<TrailBaseSession> = LocalStorage::get(SESSION_KEY).ok();
    if session.is_some() {
        cache_write(session.clone());
    }
    session
}

pub fn set_session(session: &TrailBaseSession) -> Result<(), String> {
    LocalStorage::set(SESSION_KEY, session).map_err(|e| format!("Failed to set session: {}", e))?;
    cache_write(Some(session.clone()));
    Ok(())
}

pub fn clear_session() {
    LocalStorage::delete(SESSION_KEY);
    cache_write(None);
}

// ── Async API (Tauri store + cache + localStorage) ───────────────────

/// Builds a JS `{ key: <value> }` object for IPC commands that take a single
/// `key: String` parameter. The property name `"key"` must match the Tauri
/// command's Rust parameter name.
fn make_key_arg(key: &str) -> JsValue {
    let obj = Object::new();
    let _ = Reflect::set(&obj, &JsValue::from_str("key"), &JsValue::from_str(key));
    obj.into()
}

/// Builds a JS `{ key: <key>, value: <value> }` object for the
/// `auth_store_set` command which takes `key: String, value: String`.
fn make_key_value_arg(key: &str, value: &str) -> JsValue {
    let obj = Object::new();
    let _ = Reflect::set(&obj, &JsValue::from_str("key"), &JsValue::from_str(key));
    let _ = Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::from_str(value));
    obj.into()
}

async fn store_read(key: &str) -> Option<String> {
    let args = make_key_arg(key);
    match tauri::invoke_with_args("auth_store_get", &args).await {
        Ok(v) if !v.is_null() && !v.is_undefined() => v.as_string(),
        Ok(_) => None,
        Err(e) => {
            warn!("store_read('{}') IPC error: {}", key, e);
            None
        },
    }
}

async fn store_write(key: &str, value: &str) -> Result<(), String> {
    let args = make_key_value_arg(key, value);
    tauri::invoke_with_args("auth_store_set", &args)
        .await
        .map(|_| ())
}

async fn store_delete(key: &str) -> Result<(), String> {
    let args = make_key_arg(key);
    tauri::invoke_with_args("auth_store_delete", &args)
        .await
        .map(|_| ())
}

/// Reads the session, populating the cache from the persistent store on the
/// first cold-path access (Tauri) or from localStorage (web build).
pub async fn get_session_async() -> Option<TrailBaseSession> {
    if let Some(cached) = cache_read() {
        return Some(cached);
    }

    if tauri::is_tauri() {
        let session = store_read(SESSION_KEY)
            .await
            .and_then(|json| serde_json::from_str::<TrailBaseSession>(&json).ok());
        if let Some(ref s) = session {
            debug!("session_loaded_from_store: email={}", s.email);
            cache_write(session.clone());
        }
        session
    } else {
        get_session()
    }
}

/// Writes the session to the persistent store (Tauri) or localStorage (web),
/// then updates the cache.
pub async fn set_session_async(session: &TrailBaseSession) -> Result<(), String> {
    if tauri::is_tauri() {
        let json = serde_json::to_string(session).map_err(|e| e.to_string())?;
        store_write(SESSION_KEY, &json).await?;
        debug!("session_write_to_store: email={}", session.email);
    }
    set_session(session)?;
    Ok(())
}

/// Clears the session from the persistent store (Tauri) or localStorage (web),
/// then clears the cache.
pub async fn clear_session_async() {
    if tauri::is_tauri()
        && let Err(e) = store_delete(SESSION_KEY).await
    {
        warn!("Failed to clear session from Tauri store: {}", e);
    }
    debug!("session_clear: cache + localStorage cleared");
    clear_session();
}

// ── PKCE verifier persistence ────────────────────────────────────────
//
// The PKCE verifier must survive the process kill between the OAuth redirect
// (external browser) and the deep-link callback (app cold start), otherwise
// the authorization code cannot be exchanged for a token.

pub async fn set_pkce_verifier_async(verifier: &str) -> Result<(), String> {
    if tauri::is_tauri() {
        store_write(PKCE_VERIFIER_KEY, verifier).await?;
    }
    LocalStorage::set(PKCE_VERIFIER_KEY, verifier).map_err(|e| e.to_string())?;
    Ok(())
}

/// Atomically reads and deletes the PKCE verifier from both the store and
/// localStorage. Returns `None` if not found in either.
pub async fn take_pkce_verifier_async() -> Option<String> {
    let verifier = if tauri::is_tauri() {
        store_read(PKCE_VERIFIER_KEY).await
    } else {
        LocalStorage::get::<String>(PKCE_VERIFIER_KEY).ok()
    };

    if verifier.is_some() {
        if tauri::is_tauri()
            && let Err(e) = store_delete(PKCE_VERIFIER_KEY).await
        {
            warn!("Failed to delete PKCE verifier from store: {}", e);
        }
        LocalStorage::delete(PKCE_VERIFIER_KEY);
    }

    verifier
}

// ── One-time migration from localStorage to store ────────────────────

/// Migrates the session from localStorage to the Tauri store if the store is
/// empty but localStorage has a session. Called once on app init (Tauri only).
/// After a successful migration, the localStorage copy is deleted so the store
/// becomes the single persistent source. If serialization or store write fails,
/// the localStorage copy is preserved.
pub async fn migrate_session_to_store_if_needed() {
    if !tauri::is_tauri() {
        return;
    }

    if store_read(SESSION_KEY).await.is_some() {
        return;
    }

    let local_session: Option<TrailBaseSession> = LocalStorage::get(SESSION_KEY).ok();
    let Some(session) = local_session else {
        return;
    };

    let json = match serde_json::to_string(&session) {
        Ok(j) => j,
        Err(e) => {
            warn!("Session serialization for migration failed: {}", e);
            return;
        },
    };

    if let Err(e) = store_write(SESSION_KEY, &json).await {
        warn!("Session migration to store failed: {}", e);
        return;
    }

    LocalStorage::delete(SESSION_KEY);
}

// ── Non-session localStorage helpers (unchanged) ─────────────────────

const LAST_SYNC_KEY: &str = "origa_last_sync_time";

pub fn set_last_sync_time(timestamp: u64) {
    let res = LocalStorage::set(LAST_SYNC_KEY, timestamp);
    if let Err(e) = res {
        tracing::error!("Failed to set last sync time: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_round_trip() {
        cache_write(None);
        assert!(cache_read().is_none());

        let session = TrailBaseSession {
            auth_token: "token".to_string(),
            refresh_token: "refresh".to_string(),
            email: "test@example.com".to_string(),
            trailbase_id: "id".to_string(),
            record_id: None,
            expires_at: 0,
        };
        cache_write(Some(session.clone()));
        assert_eq!(cache_read().unwrap().email, session.email);

        cache_write(None);
        assert!(cache_read().is_none());
    }
}
