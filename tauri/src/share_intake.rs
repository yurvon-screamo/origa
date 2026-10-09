//! Share intake bridge (IN-2/IN-3): delivers externally shared content
//! (Android share sheet, desktop file association) into the inbox.
//!
//! Files never travel through events: the host writes the bytes to
//! `cache_dir()/share-intake/<uuid>.<ext>` (Kotlin writes the same
//! subdirectory on Android) and emits only metadata + the cache path; the
//! frontend reads the bytes with [`read_share_file`] (one invoke cycle,
//! the `pick_and_read_file` precedent) and deletes the file afterwards.
//! Cold starts are covered by a pending slot the frontend polls
//! (`get_pending_share`, the `get_current_deep_link` precedent).

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

/// Upper bound for one shared file — matches the audio pipeline cap.
const MAX_FILE_BYTES: u64 = 200 * 1024 * 1024;
/// Cache subdirectory shared with the Kotlin writer (Android).
pub(crate) const SHARE_INTAKE_DIR: &str = "share-intake";
/// Event name delivered to the main window.
pub(crate) const SHARE_INTAKE_EVENT: &str = "origa://share-intake";

/// Wire payload: text or a file parked in the cache directory.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum ShareWire {
    #[default]
    None,
    Text {
        text: String,
    },
    File {
        file_name: String,
        mime: String,
        /// Path under `cache_dir()/share-intake/` (host-written).
        cache_path: String,
    },
    /// The share could not be captured (size cap, unreadable source).
    Error {
        message: String,
    },
}

/// Host-side pending slot (cold-start delivery).
static PENDING: std::sync::Mutex<Option<ShareWire>> = std::sync::Mutex::new(None);

/// Whether the frontend share listener has mounted (set by the
/// `share_listener_ready` command). `Opened` events before this point
/// are cold — their emit goes nowhere, so the pending slot is the only
/// reliable channel; after this point, emit is warm and pending would
/// just become a stale duplicate.
static LISTENER_READY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Marks the frontend share listener as mounted. Call once from the
/// frontend after `start_share_intake`.
#[tauri::command]
pub fn share_listener_ready() {
    LISTENER_READY.store(true, std::sync::atomic::Ordering::Release);
}

/// Whether the frontend listener has signalled readiness. Only consumed
/// by the Apple/mobile Opened handler (cfg-gated in lib.rs).
#[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
pub(crate) fn is_listener_ready() -> bool {
    LISTENER_READY.load(std::sync::atomic::Ordering::Acquire)
}

fn store_pending(payload: &ShareWire) {
    let mut pending = PENDING.lock().expect("share-intake pending lock");
    *pending = Some(payload.clone());
}

/// Returns and clears the pending share (take-once; a newer share
/// overwrites an unconsumed older one).
#[tauri::command]
pub fn get_pending_share() -> Option<ShareWire> {
    // Android: the Kotlin ShareBuffer owns the cold-start/warm shares;
    // drain it first (JNI), then fall back to the desktop pending slot.
    #[cfg(target_os = "android")]
    if let Some(wire) = take_android_pending() {
        return Some(wire);
    }
    let mut pending = PENDING.lock().expect("share-intake pending lock");
    pending.take()
}

/// Drains the Kotlin-side ShareBuffer over JNI (ADR-044: the JavaVM comes
/// from the single ndk-context publisher; JNI failures degrade to None —
/// no panics across the FFI boundary, release builds abort on panic).
#[cfg(target_os = "android")]
fn take_android_pending() -> Option<ShareWire> {
    let vm = crate::android_context::java_vm()?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| {
            tracing::warn!("[share-intake] JNI thread attach failed: {e:?}");
            e
        })
        .ok()?;

    // FindClass from an attached native thread uses the system class
    // loader, which cannot see app classes — resolve ShareBuffer through
    // the Application's classLoader instead (JNI Tips: class resolution
    // off the Java stack).
    let app_context = crate::android_context::app_context()?;
    let class_loader = env
        .call_method(
            app_context,
            "getClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )
        .map_err(|e| {
            tracing::warn!("[share-intake] getClassLoader failed: {e:?}");
            e
        })
        .ok()?
        .l()
        .map_err(|e| {
            tracing::warn!("[share-intake] getClassLoader returned non-object: {e:?}");
            e
        })
        .ok()?;

    let class_name = env
        .new_string("net.uwuwu.origa.ShareBuffer")
        .map_err(|e| {
            tracing::warn!("[share-intake] new_string failed: {e:?}");
            e
        })
        .ok()?;
    let class = env
        .call_static_method(
            "java/lang/Class",
            "forName",
            "(Ljava/lang/String;Ljava/lang/ClassLoader;)Ljava/lang/Class;",
            &[
                jni::objects::JValue::Object(&class_name),
                jni::objects::JValue::Object(&class_loader),
            ],
        )
        .map_err(|e| {
            tracing::warn!("[share-intake] Class.forName failed: {e:?}");
            e
        })
        .ok()?
        .l()
        .map_err(|e| {
            tracing::warn!("[share-intake] forName returned non-object: {e:?}");
            e
        })
        .ok()?;

    let class: jni::objects::JClass = class.into();
    let json_value = env
        .call_static_method(&class, "takePending", "()Ljava/lang/String;", &[])
        .map_err(|e| {
            tracing::warn!("[share-intake] takePending JNI call failed: {e:?}");
            e
        })
        .ok()?
        .l()
        .map_err(|e| {
            tracing::warn!("[share-intake] takePending returned non-object: {e:?}");
            e
        })
        .ok()?;
    // Ownership transfer: JObject::into() → JString consumes the local
    // ref (a from_raw + alive JObject would double-delete on drop).
    let jstring: jni::objects::JString = json_value.into();
    let json: String = env
        .get_string(&jstring)
        .map_err(|e| {
            tracing::warn!("[share-intake] JNI string conversion failed: {e:?}");
            e
        })
        .ok()?
        .into();
    serde_json::from_str::<ShareWire>(&json)
        .map_err(|e| {
            tracing::warn!("[share-intake] ShareBuffer JSON decode failed: {e:?}");
            e
        })
        .ok()
}

/// Warm delivery: emits to the mounted frontend listener. Cold-start
/// callers (argv before the window exists, pre-run-loop ingests) use
/// [`store_pending_and_emit`] instead — `emit` returns Ok even with no
/// listener, so cold/warm is decided at the call site, not from the
/// emit result.
pub(crate) fn emit_share(app: &AppHandle, payload: ShareWire) {
    if let Err(e) = app.emit(SHARE_INTAKE_EVENT, &payload) {
        tracing::warn!("[share-intake] emit failed: {e:?}");
    }
}

/// Cold-start delivery: stores into the pending slot (the frontend poll
/// drains it on mount) AND emits (a fast warm listener may win the race).
pub(crate) fn store_pending_and_emit(app: &AppHandle, payload: ShareWire) {
    store_pending(&payload);
    emit_share(app, payload);
}

/// Writes shared bytes into the cache directory and returns the wire
/// payload for them. `extension` should be empty or dot-less ("png").
pub(crate) fn park_shared_bytes(
    app: &AppHandle,
    bytes: Vec<u8>,
    file_name: &str,
    mime: &str,
    extension: &str,
) -> ShareWire {
    let dir = app
        .path()
        .app_cache_dir()
        .map(|base| base.join(SHARE_INTAKE_DIR));
    let dir = match dir {
        Ok(dir) => dir,
        Err(e) => {
            return ShareWire::Error {
                message: format!("Cache directory unavailable: {e}"),
            };
        },
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return ShareWire::Error {
            message: format!("Cache directory creation failed: {e}"),
        };
    }
    let unique_name = match extension.is_empty() {
        true => Uuid::new_v4().to_string(),
        false => format!("{}.{}", Uuid::new_v4(), extension),
    };
    let path = dir.join(unique_name);
    if let Err(e) = std::fs::write(&path, &bytes) {
        return ShareWire::Error {
            message: format!("Shared file write failed: {e}"),
        };
    }
    ShareWire::File {
        file_name: file_name.to_string(),
        mime: mime.to_string(),
        cache_path: path.to_string_lossy().to_string(),
    }
}

/// The canonical cache subdirectory; the read/delete commands accept only
/// paths inside it.
fn share_intake_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_cache_dir()
        .map(|base| base.join(SHARE_INTAKE_DIR))
        .map_err(|e| format!("Cache directory unavailable: {e}"))
}

/// Accepts only paths that resolve inside the share-intake directory.
/// Canonicalize first: a string-prefix check is bypassable via `..`.
fn validate_cache_path(app: &AppHandle, path: &str) -> Result<std::path::PathBuf, String> {
    let dir = share_intake_dir(app)?;
    let canonical_dir = std::fs::canonicalize(&dir)
        .map_err(|_| "Share intake directory is not available".to_string())?;
    let canonical =
        std::fs::canonicalize(path).map_err(|_| "Shared file no longer exists".to_string())?;
    if !canonical.starts_with(&canonical_dir) {
        return Err("Path is outside the share-intake directory".to_string());
    }
    Ok(canonical)
}

/// Reads a parked shared file as base64 (one invoke cycle, the
/// `pick_and_read_file` transport precedent).
#[tauri::command]
pub fn read_share_file(app: AppHandle, path: String) -> Result<String, String> {
    let canonical = validate_cache_path(&app, &path)?;
    let metadata =
        std::fs::metadata(&canonical).map_err(|e| format!("Cannot read the shared file: {e}"))?;
    if metadata.len() > MAX_FILE_BYTES {
        return Err(format!(
            "The shared file exceeds the {} MB limit",
            MAX_FILE_BYTES / (1024 * 1024)
        ));
    }
    let bytes =
        std::fs::read(&canonical).map_err(|e| format!("Cannot read the shared file: {e}"))?;
    Ok(STANDARD.encode(bytes))
}

/// Deletes a consumed shared file. Best-effort: a leftover is swept on
/// the next startup cleanup.
#[tauri::command]
pub fn delete_share_file(app: AppHandle, path: String) -> Result<(), String> {
    let canonical = validate_cache_path(&app, &path)?;
    std::fs::remove_file(&canonical).map_err(|e| format!("Cannot delete the shared file: {e}"))
}

/// Removes leftovers from crashed sessions (called once at startup).
/// Only sweeps files older than STALE_AFTER_SECS: on Android the Kotlin
/// writer parks the cold-start share in `onCreate`, racing this sweep —
/// a fresh file must survive.
pub(crate) fn cleanup_stale_share_files(app: &AppHandle) {
    const STALE_AFTER_SECS: u64 = 3600;
    let Ok(dir) = share_intake_dir(app) else {
        return;
    };
    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(STALE_AFTER_SECS))
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let is_stale = entry
                .metadata()
                .and_then(|meta| meta.modified())
                .is_ok_and(|modified| modified < cutoff);
            if is_stale {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

/// Parses launch arguments into existing file paths. Deep-link URLs
/// (`origa://…`, owned by the deep-link plugin) and flags are skipped.
pub(crate) fn parse_shared_paths(argv: &[String]) -> Vec<std::path::PathBuf> {
    argv.iter()
        .filter(|arg| !arg.starts_with("origa://") && !arg.starts_with('-'))
        .map(std::path::PathBuf::from)
        .filter(|path| path.is_file())
        .collect()
}

/// Extension without the dot for a shared file name ("" when absent).
pub(crate) fn extension_of(file_name: &str) -> String {
    std::path::Path::new(file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// Reads a shared file from disk and emits it through the bridge.
/// Applies the size cap and derives the extension from the file name.
pub(crate) fn ingest_path(app: &AppHandle, path: &std::path::Path) {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(e) => {
            emit_share(
                app,
                ShareWire::Error {
                    message: format!("Cannot read the shared file: {e}"),
                },
            );
            return;
        },
    };
    if metadata.len() > MAX_FILE_BYTES {
        emit_share(
            app,
            ShareWire::Error {
                message: format!(
                    "The shared file exceeds the {} MB limit",
                    MAX_FILE_BYTES / (1024 * 1024)
                ),
            },
        );
        return;
    }
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            emit_share(
                app,
                ShareWire::Error {
                    message: format!("Cannot read the shared file: {e}"),
                },
            );
            return;
        },
    };
    let payload = park_shared_bytes(app, bytes, &file_name, "", &extension_of(&file_name));
    // Warm path by default (single-instance and Opened-during-run both
    // arrive with a live window); the cold-start argv caller overrides
    // with store_pending_and_emit after this returns.
    emit_share(app, payload);
}

/// Parses argv and ingests every existing file path (warm single-instance
/// delivery on Windows/Linux).
#[cfg(any(windows, target_os = "linux"))]
pub(crate) fn ingest_argv(app: &AppHandle, args: &[String]) {
    for path in parse_shared_paths(args) {
        ingest_path(app, &path);
    }
}

/// Cold-start variant: the pending slot carries the payload (the frontend
/// drains it on mount) in addition to the best-effort emit.
pub(crate) fn ingest_path_cold(app: &AppHandle, path: &std::path::Path) {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let Ok(bytes) = std::fs::read(path) else {
        return;
    };
    let payload = park_shared_bytes(app, bytes, &file_name, "", &extension_of(&file_name));
    store_pending_and_emit(app, payload);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire() -> ShareWire {
        ShareWire::Text {
            text: "本を読む".to_string(),
        }
    }

    #[test]
    fn pending_slot_is_take_once_and_last_write_wins() {
        let mut pending = PENDING.lock().expect("test lock");
        assert!(pending.is_none());
        *pending = Some(wire());
        // A newer share overwrites the unconsumed older one.
        *pending = Some(ShareWire::Text {
            text: "新しい".to_string(),
        });
        let taken = pending.take();
        assert!(
            matches!(&taken, Some(ShareWire::Text { text }) if text == "新しい"),
            "last write must win: {taken:?}"
        );
        assert!(pending.take().is_none(), "second take must be empty");
    }

    #[test]
    fn shared_paths_skip_urls_flags_and_missing_files() {
        let existing = std::env::temp_dir().join("share-intake-test-marker.txt");
        std::fs::write(&existing, b"x").expect("marker write");
        let argv = vec![
            "origa://add?source=camera".to_string(),
            "--flag".to_string(),
            existing.to_string_lossy().to_string(),
            "/nonexistent/path.png".to_string(),
        ];
        let parsed = parse_shared_paths(&argv);
        assert_eq!(parsed, vec![existing.clone()]);
        let _ = std::fs::remove_file(&existing);
    }

    #[test]
    fn share_wire_serializes_camel_case_with_tags() {
        // Adjacently tagged enums serialize as {"kind": "...", ...} with
        // camelCase fields — the exact wire format the frontend decodes.
        let json = serde_json::to_string(&wire()).expect("serialize");
        assert!(
            json.contains("本を読む"),
            "text content must survive serialization: {json}"
        );
        assert!(json.contains("\"kind\":\"text\""), "tag present: {json}");
        let file = ShareWire::File {
            file_name: "a.png".to_string(),
            mime: "image/png".to_string(),
            cache_path: "/tmp/x".to_string(),
        };
        let json = serde_json::to_string(&file).expect("serialize");
        assert!(json.contains("fileName"), "camelCase fields: {json}");
        assert!(json.contains("cachePath"), "camelCase fields: {json}");
        // Round-trip is the contract the frontend relies on.
        let round: ShareWire = serde_json::from_str(&json).expect("round-trip");
        assert!(
            matches!(round, ShareWire::File { ref file_name, ref cache_path, .. } if file_name == "a.png" && cache_path == "/tmp/x")
        );
    }

    #[test]
    fn extensions_parse_lowercased() {
        assert_eq!(extension_of("photo.PNG"), "png");
        assert_eq!(extension_of("photo"), "");
        assert_eq!(extension_of("archive.tar.gz"), "gz");
    }
}
