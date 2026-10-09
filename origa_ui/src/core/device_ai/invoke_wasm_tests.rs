//! WASM tests for the `vision_recognize_text` payload contract.
//!
//! The plugin splits language selection by platform: iOS/macOS/Windows read
//! `options.languages`, Android ML Kit reads `options.script` and defaults to
//! the Latin recognizer when it is absent — which silently broke Japanese OCR
//! on Android. These tests pin the payload so the cross-platform fields
//! cannot regress independently.

#![cfg(all(target_arch = "wasm32", test))]

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use super::invoke::build_recognize_text_payload;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn recognize_text_payload_sets_japanese_script_for_android_ml_kit() {
    let payload = build_recognize_text_payload("aGVsbG8=");

    let options = js_sys::Reflect::get(&payload, &JsValue::from_str("options"))
        .expect("payload.options exists");

    let script = js_sys::Reflect::get(&options, &JsValue::from_str("script"))
        .expect("options.script exists");

    assert_eq!(script.as_string().as_deref(), Some("japanese"));
}

#[wasm_bindgen_test]
fn recognize_text_payload_keeps_vision_fields_for_apple_platforms() {
    let payload = build_recognize_text_payload("aGVsbG8=");

    let options = js_sys::Reflect::get(&payload, &JsValue::from_str("options"))
        .expect("payload.options exists");
    let languages = js_sys::Reflect::get(&options, &JsValue::from_str("languages"))
        .expect("options.languages exists");
    let languages = languages
        .dyn_into::<js_sys::Array>()
        .expect("options.languages is an array");
    assert_eq!(languages.length(), 1);
    assert_eq!(
        languages.at(0).as_string().as_deref(),
        Some("ja"),
        "Japanese BCP-47 tag for Vision recognitionLanguages"
    );

    let level = js_sys::Reflect::get(&options, &JsValue::from_str("recognitionLevel"))
        .expect("options.recognitionLevel exists");
    assert_eq!(level.as_string().as_deref(), Some("accurate"));

    let image =
        js_sys::Reflect::get(&payload, &JsValue::from_str("image")).expect("payload.image exists");
    let base64 =
        js_sys::Reflect::get(&image, &JsValue::from_str("base64")).expect("image.base64 exists");
    assert_eq!(base64.as_string().as_deref(), Some("aGVsbG8="));
}
