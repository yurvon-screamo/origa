//! WASM-only inbox tests: branches that construct or inspect browser
//! objects (`web_sys::File`, `window` properties, `localStorage`).

use super::seam::{InboxSeamGuard, SEAM_FLAG_STORAGE_KEY, SEAM_PROPERTY, register_inbox_seam_with};
use super::{InboxKind, InboxPayload, InboxRoute, route_payload};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn make_file(name: &str, mime: &str) -> web_sys::File {
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::new_with_length(16).into());
    let bag = web_sys::FilePropertyBag::new();
    web_sys::FilePropertyBag::set_type(&bag, mime);
    web_sys::File::new_with_str_sequence_and_options(&parts.into(), name, &bag)
        .expect("File construction must succeed in a browser context")
}

/// Clears the seam opt-in flag so tests never leak the key into each other
/// (wasm-bindgen-test shares one browser session across the suite).
fn set_seam_flag(value: Option<&str>) {
    let window = web_sys::window().expect("window must exist");
    let storage = window
        .local_storage()
        .expect("localStorage must be accessible")
        .expect("storage must be present");
    match value {
        Some(v) => {
            storage.set_item(SEAM_FLAG_STORAGE_KEY, v).ok();
        },
        None => {
            storage.remove_item(SEAM_FLAG_STORAGE_KEY).ok();
        },
    };
}

fn dummy_closure() -> Closure<dyn Fn(JsValue)> {
    Closure::wrap(Box::new(|_: JsValue| {}) as Box<dyn Fn(JsValue)>)
}

fn seam_property_value() -> JsValue {
    let window = web_sys::window().expect("window must exist");
    js_sys::Reflect::get(&window.into(), &JsValue::from_str(SEAM_PROPERTY))
        .expect("property read must not throw")
}

#[wasm_bindgen_test]
fn file_payload_routes_by_class() {
    let image = route_payload(InboxPayload {
        kind: InboxKind::File(make_file("homework.jpg", "image/jpeg")),
    });
    assert!(matches!(image, InboxRoute::OcrFile(_)));

    let audio = route_payload(InboxPayload {
        kind: InboxKind::File(make_file("lesson.m4a", "audio/mp4")),
    });
    assert!(matches!(audio, InboxRoute::SttFile(_)));

    let unsupported = route_payload(InboxPayload {
        kind: InboxKind::File(make_file("archive.zip", "application/zip")),
    });
    match unsupported {
        InboxRoute::Unsupported { mime, name } => {
            assert_eq!(mime, "application/zip");
            assert_eq!(name, "archive.zip");
        },
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

#[wasm_bindgen_test]
fn seam_registration_requires_the_opt_in_flag() {
    // Without the flag: no guard, and — critically — no window property a
    // page script could call.
    set_seam_flag(None);
    let registered = register_inbox_seam_with(dummy_closure());
    assert!(
        registered.is_none(),
        "seam must stay dormant without the flag"
    );
    assert!(
        seam_property_value().is_undefined(),
        "property must stay absent"
    );

    // With the exact opt-in value: the property appears.
    set_seam_flag(Some("1"));
    let guard: Option<InboxSeamGuard> = register_inbox_seam_with(dummy_closure());
    assert!(guard.is_some(), "seam must register with the opt-in flag");
    assert!(
        !seam_property_value().is_undefined(),
        "property must be registered"
    );

    // Dropping the guard removes the property (dispose discipline).
    drop(guard);
    assert!(
        seam_property_value().is_undefined(),
        "property must be removed on drop"
    );
    set_seam_flag(None);
}
