//! Native file picking (desktop Tauri only).
//!
//! A single host command opens the native dialog and reads the picked file
//! (`pick_and_read_file`) — the webview supplies only filter metadata, the
//! path never crosses the trust boundary. On web and mobile the picker is
//! unavailable and the WebView file input remains the path.

use wasm_bindgen::{JsCast, JsValue};

use super::tauri::{invoke_with_args, is_tauri};

/// What kind of file a picker button accepts. Drives the dialog filter and
/// the MIME type assigned to the constructed `File`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKind {
    Image,
    AnkiDeck,
}

impl PickerKind {
    /// (filter name, extensions) for the host dialog.
    pub fn dialog_filter(&self) -> (&'static str, &'static [&'static str]) {
        match self {
            PickerKind::Image => ("Images", &["png", "jpg", "jpeg", "webp"]),
            PickerKind::AnkiDeck => ("Anki deck", &["apkg"]),
        }
    }

    /// MIME type for the constructed `File`, derived from the picked file
    /// name. Deriving (not hardcoding) matters: the OCR pipeline converts
    /// the file through FileReader, whose data-URL prefix is taken from
    /// this type — a wrong or empty type breaks the `data:image/` parse
    /// even when the bytes are fine. Unknown extensions keep an empty type
    /// (reachable only via the dialog's All-files bypass): `is_image_file`
    /// admits empty, so such a file fails later at the data-URL parse with
    /// a raw error — honest, not localized.
    pub fn mime_for_name(&self, name: &str) -> &'static str {
        let lower = name.to_ascii_lowercase();
        match self {
            PickerKind::AnkiDeck => "application/octet-stream",
            PickerKind::Image => {
                if lower.ends_with(".png") {
                    "image/png"
                } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
                    "image/jpeg"
                } else if lower.ends_with(".webp") {
                    "image/webp"
                } else {
                    ""
                }
            },
        }
    }
}

/// Pure visibility predicate: the native picker exists only in the desktop
/// Tauri shell with the dialog JS bridge injected. Web keeps the WebView
/// file input; mobile keeps it too (content:// and security-scoped URLs are
/// not serviceable by the host read command).
pub fn native_picker_available(
    is_tauri_shell: bool,
    platform: &str,
    dialog_bridge_present: bool,
) -> bool {
    let mobile = matches!(platform, "android" | "ios");
    is_tauri_shell && !mobile && dialog_bridge_present
}

/// Whether the native picker button should render on this runtime.
pub fn is_available() -> bool {
    let shell = is_tauri();
    if !shell {
        return false;
    }
    let platform = crate::core::platform::platform_name();
    let dialog_bridge = crate::core::tauri::tauri_object().is_some_and(|obj| {
        js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str("dialog"))
            .is_ok_and(|v| !v.is_undefined() && !v.is_null())
    });
    native_picker_available(shell, &platform, dialog_bridge)
}

/// Opens the native dialog and returns (file name, decoded bytes) for the
/// picked file, or `None` when the user cancelled.
pub async fn pick_and_read_file(kind: PickerKind) -> Result<Option<(String, Vec<u8>)>, String> {
    let (filter_name, extensions) = kind.dialog_filter();
    let extensions: Vec<JsValue> = extensions
        .iter()
        .map(|ext| JsValue::from_str(ext))
        .collect();
    let args = js_sys::Object::new();
    let payload = js_sys::Object::new();
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("filter_name"),
        &JsValue::from_str(filter_name),
    )
    .map_err(|_| "dialog args: filter_name".to_string())?;
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("extensions"),
        &js_sys::Array::from_iter(extensions).into(),
    )
    .map_err(|_| "dialog args: extensions".to_string())?;
    js_sys::Reflect::set(&args, &JsValue::from_str("args"), &payload.into())
        .map_err(|_| "dialog args: payload".to_string())?;

    let raw = invoke_with_args("pick_and_read_file", &args.into()).await?;
    let raw_object: js_sys::Object = raw
        .dyn_into()
        .map_err(|_| "dialog response is not an object".to_string())?;
    let payload = js_sys::Reflect::get(&raw_object, &JsValue::from_str("payload"))
        .map_err(|_| "dialog response: payload".to_string())?;
    if payload.is_null() || payload.is_undefined() {
        return Ok(None); // user cancelled
    }
    let name = js_sys::Reflect::get(&payload, &JsValue::from_str("name"))
        .map_err(|_| "dialog response: name".to_string())?
        .as_string()
        .ok_or("dialog response: name is not a string")?;
    let base64 = js_sys::Reflect::get(&payload, &JsValue::from_str("base64"))
        .map_err(|_| "dialog response: base64".to_string())?
        .as_string()
        .ok_or("dialog response: base64 is not a string")?;

    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        base64.as_bytes(),
    )
    .map_err(|e| format!("dialog response: base64 decode failed: {e:?}"))?;
    Ok(Some((name, bytes)))
}
