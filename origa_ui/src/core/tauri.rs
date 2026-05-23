//! Unified access to Tauri v2 JS API (`window.__TAURI__`).
//!
//! All Tauri platform detection and JS API access should go through this module.
//! No other file should use `js_sys::Reflect::get(...("__TAURI__"))` directly.

use js_sys::{Function, Object, Reflect};
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use web_sys::window;

/// Returns `true` if running inside a Tauri WebView (desktop or mobile).
pub fn is_tauri() -> bool {
    tauri_object().is_some()
}

/// Returns the `window.__TAURI__` object if available.
pub fn tauri_object() -> Option<Object> {
    window()
        .and_then(|w| Reflect::get(&w, &JsValue::from_str("__TAURI__")).ok())
        .and_then(|v| {
            if v.is_undefined() || v.is_null() {
                None
            } else {
                v.dyn_into::<Object>().ok()
            }
        })
}

/// Returns `window.__TAURI__.core.invoke` function if available.
///
/// Used to call Tauri commands from WASM.
pub fn invoke_fn() -> Option<Function> {
    let obj = tauri_object()?;
    let core = Reflect::get(&obj, &JsValue::from_str("core")).ok()?;
    Reflect::get(&core, &JsValue::from_str("invoke"))
        .ok()
        .and_then(|v| v.dyn_into::<Function>().ok())
}

/// Returns `window.__TAURI__.event.listen` function if available.
///
/// Used to register event listeners from WASM.
pub fn event_listen_fn() -> Option<Function> {
    let obj = tauri_object()?;
    let event = Reflect::get(&obj, &JsValue::from_str("event")).ok()?;
    Reflect::get(&event, &JsValue::from_str("listen"))
        .ok()
        .and_then(|v| v.dyn_into::<Function>().ok())
}

/// Returns `window.__TAURI__.opener.openUrl` function if available.
///
/// Used to open URLs in the system browser from Tauri.
pub fn opener_open_url_fn() -> Option<Function> {
    let obj = tauri_object()?;
    let opener = Reflect::get(&obj, &JsValue::from_str("opener")).ok()?;
    Reflect::get(&opener, &JsValue::from_str("openUrl"))
        .ok()
        .and_then(|v| v.dyn_into::<Function>().ok())
}
