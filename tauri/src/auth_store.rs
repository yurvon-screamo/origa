//! Tauri IPC commands for persistent key-value storage via `tauri-plugin-store`.
//!
//! The frontend (WASM) calls these commands through `invoke` to persist the
//! TrailBase session and PKCE verifier to the native filesystem
//! (`app_data_dir/auth.json`). This is required on Android where the WebView
//! `localStorage` is unreliable under process kills (Chromium DOMStorage commit
//! queue, see ADR-010).
//!
//! Every write command calls `Store::save()` explicitly to fsync to disk before
//! returning, because the plugin's default `auto_save` debounce (100 ms) is
//! insufficient when the OS can kill the process at any moment.

use tauri::AppHandle;
use tauri_plugin_store::{JsonValue, StoreExt};

const STORE_FILE: &str = "auth.json";

/// Extracts the inner `String` from a `JsonValue`.
///
/// `Store::get` returns `Option<JsonValue>` (a re-export of `serde_json::Value`).
/// When a `String` is stored via `Store::set`, it becomes `Value::String(s)`.
/// Using `Value::to_string()` here would **double-encode** the value (it
/// serializes back to JSON, wrapping the string in quotes and escaping inner
/// quotes). Using `as_str()` extracts the original unencoded string.
fn extract_string(value: JsonValue) -> Option<String> {
    value.as_str().map(|s| s.to_string())
}

fn read_value(app: &AppHandle, key: &str) -> Result<Option<String>, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    Ok(store.get(key).and_then(extract_string))
}

fn write_value(app: &AppHandle, key: &str, value: &str) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(key, value.to_string());
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

fn delete_value(app: &AppHandle, key: &str) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.delete(key);
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Reads a string value for `key` from the persistent store.
///
/// Returns `None` if the key does not exist.
#[tauri::command]
pub fn auth_store_get(app: AppHandle, key: String) -> Result<Option<String>, String> {
    read_value(&app, &key)
}

/// Writes `value` for `key` and fsyncs to disk before returning.
#[tauri::command]
pub fn auth_store_set(app: AppHandle, key: String, value: String) -> Result<(), String> {
    write_value(&app, &key, &value)
}

/// Deletes `key` from the store and fsyncs to disk before returning.
#[tauri::command]
pub fn auth_store_delete(app: AppHandle, key: String) -> Result<(), String> {
    delete_value(&app, &key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_string_from_json_string_value() {
        let value = JsonValue::String("hello_world".to_string());
        assert_eq!(extract_string(value), Some("hello_world".to_string()));
    }

    #[test]
    fn extract_string_from_json_object_value() {
        // A serialized session JSON stored as a String variant.
        let json = r#"{"auth_token":"xyz","refresh_token":"abc"}"#;
        let value = JsonValue::String(json.to_string());
        let extracted = extract_string(value);
        assert_eq!(extracted.as_deref(), Some(json));

        // The extracted string must round-trip through serde_json without
        // double-encoding (this is the regression that H1 fixed).
        let parsed: serde_json::Value =
            serde_json::from_str(&extracted.unwrap()).expect("must parse as JSON object");
        assert_eq!(parsed["auth_token"], "xyz");
    }

    #[test]
    fn extract_string_returns_none_for_non_string_value() {
        let value = JsonValue::Bool(true);
        assert_eq!(extract_string(value), None);
    }

    #[test]
    fn extract_string_does_not_double_encode() {
        // Simulates the exact failure mode: if to_string() were used instead
        // of as_str(), a Value::String containing JSON would be wrapped in
        // extra quotes, and parsing it back would yield a JSON string, not an
        // object.
        let json = r#"{"key":"value"}"#;
        let value = JsonValue::String(json.to_string());

        // Correct behavior (as_str): extracts the raw string.
        let extracted = extract_string(value).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&extracted).unwrap();
        assert!(parsed.is_object(), "extracted string must be a JSON object");

        // Incorrect behavior (to_string): double-encodes.
        let double_encoded = JsonValue::String(json.to_string()).to_string();
        let parsed_wrong: serde_json::Value = serde_json::from_str(&double_encoded).unwrap();
        assert!(
            parsed_wrong.is_string(),
            "to_string() produces a JSON string, not object"
        );
    }
}
