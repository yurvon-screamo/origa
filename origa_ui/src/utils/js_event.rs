//! Defensive reads of string fields from DOM events.
//!
//! `web_sys` declares `KeyboardEvent.code` / `key` as non-optional `String`,
//! but Android WebView can deliver a `keydown` whose event object has neither
//! field. The generated wasm-bindgen getter then hands `undefined` to
//! `passStringToWasm0`, which throws `TypeError: ... reading 'length'` out of
//! the listener (uwuwu#1702, RUST-7). Read the property through reflection
//! instead so a missing or non-string value stays data, not a trap.

use js_sys::Reflect;
use wasm_bindgen::JsValue;

/// Returns the event's string field, or an empty string when it is missing
/// or not a string. Empty is a safe non-match for every keyboard predicate.
pub(crate) fn event_string_field(event: &JsValue, field: &str) -> String {
    Reflect::get(event, &JsValue::from_str(field))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_default()
}

/// Returns the event's boolean field, or `false` when it is missing or not a
/// boolean. A keydown without `ctrlKey` / `metaKey` must not trap the guard.
pub(crate) fn event_bool_field(event: &JsValue, field: &str) -> bool {
    Reflect::get(event, &JsValue::from_str(field))
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}
