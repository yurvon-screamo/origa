//! Native file picking + reading in one host command.
//!
//! The webview sends only filter metadata; the host opens the native dialog
//! itself and reads the chosen file. The file path never crosses the trust
//! boundary, so there is no arbitrary-path-read surface — the user's
//! explicit pick in the native dialog is what authorizes the read.
//!
//! The picker BUTTON is desktop-only (mobile keeps the WebView file
//! input: content:// URIs and security-scoped URLs are not serviceable by
//! `std::fs::read`). The command itself registers on every platform — a
//! call from mobile fails gracefully at `into_path`.

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// Hard cap for a picked file. The button is exposed for images and Anki
/// decks only (small files); audio stays on the WebView file input because
/// the chunked pipeline is memory-tolerant but this IPC is not.
const MAX_FILE_BYTES: u64 = 200 * 1024 * 1024;
const MAX_FILTERS: usize = 8;
const MAX_FILTER_NAME_LEN: usize = 64;
const MAX_EXTENSION_LEN: usize = 16;

#[derive(Deserialize)]
pub struct PickAndReadArgs {
    /// Human-readable filter name, e.g. "Images".
    pub filter_name: String,
    /// Extensions WITHOUT the leading dot, e.g. ["png", "jpg"].
    pub extensions: Vec<String>,
}

#[derive(Serialize)]
pub struct FilePayload {
    /// File name (with extension) as reported by the dialog.
    pub name: String,
    /// Full file contents, standard base64.
    pub base64: String,
}

/// Host-side normalization of webview-supplied filters: drops empty or
/// oversized entries and caps the list. Not a security boundary (the user
/// still picks the file visually) — just keeps garbage filters out of the
/// native dialog.
fn normalize_extensions(extensions: &[String]) -> Vec<String> {
    extensions
        .iter()
        .map(|ext| ext.trim().to_ascii_lowercase())
        .filter(|ext| !ext.is_empty() && ext.len() <= MAX_EXTENSION_LEN)
        .take(MAX_FILTERS)
        .collect()
}

#[tauri::command]
pub async fn pick_and_read_file(
    app: AppHandle,
    args: PickAndReadArgs,
) -> Result<Option<FilePayload>, String> {
    let extensions = normalize_extensions(&args.extensions);
    if extensions.is_empty() {
        return Err("No file extensions supplied for the dialog filter".to_string());
    }
    // Char-boundary-safe truncation: byte slicing on a multibyte name
    // would panic inside the trust-boundary command.
    let filter_name = if args.filter_name.chars().count() > MAX_FILTER_NAME_LEN {
        args.filter_name.chars().take(MAX_FILTER_NAME_LEN).collect()
    } else {
        args.filter_name
    };

    // The blocking dialog must leave the async runtime's worker threads.
    tauri::async_runtime::spawn_blocking(move || {
        let extension_refs: Vec<&str> = extensions.iter().map(String::as_str).collect();
        let picked = app
            .dialog()
            .file()
            .add_filter(&filter_name, &extension_refs)
            .blocking_pick_file();
        let picked = match picked {
            Some(file) => file,
            None => return Ok(None), // user cancelled
        };
        let path = match picked.into_path() {
            Ok(path) => path,
            Err(e) => return Err(format!("Unsupported dialog result: {e:?}")),
        };
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());

        let metadata =
            std::fs::metadata(&path).map_err(|e| format!("Cannot read the picked file: {e}"))?;
        if metadata.len() > MAX_FILE_BYTES {
            return Err(format!(
                "The picked file exceeds the {} MB limit",
                MAX_FILE_BYTES / (1024 * 1024)
            ));
        }

        let bytes =
            std::fs::read(&path).map_err(|e| format!("Cannot read the picked file: {e}"))?;
        Ok(Some(FilePayload {
            name,
            base64: STANDARD.encode(bytes),
        }))
    })
    .await
    .map_err(|e| format!("File pick task failed: {e:?}"))?
}

#[cfg(test)]
mod tests {
    use super::normalize_extensions;

    #[test]
    fn normalization_trims_folds_then_caps() {
        let extensions: Vec<String> = [
            "PNG",                                            // folded to lowercase
            "  jpg  ",                                        // trimmed
            "",                                               // dropped
            "WEBP",                                           // folded
            "an-extension-that-is-way-too-long-for-a-filter", // dropped
            "gif",
            "bmp",
            "avif",
            "jxl",
            "heif", // 8th valid entry
            "tif",  // dropped by the cap (would be the 9th)
        ]
        .iter()
        .map(|ext| ext.to_string())
        .collect();
        // Filter happens BEFORE the cap: garbage entries must not eat
        // valid filter slots.
        let normalized = normalize_extensions(&extensions);
        assert_eq!(
            normalized,
            vec!["png", "jpg", "webp", "gif", "bmp", "avif", "jxl", "heif"]
        );
    }

    #[test]
    fn normalization_of_only_garbage_is_empty() {
        let extensions: Vec<String> = vec![String::new(), "   ".to_string()];
        assert!(normalize_extensions(&extensions).is_empty());
    }
}
