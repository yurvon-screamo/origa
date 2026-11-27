//! Browser-level coverage of the JS→WASM event boundary (uwuwu#1702, RUST-7).
//!
//! The listener regression suite covers a `keydown` whose string fields are
//! absent. These tests cover the neighbouring shape seen with wrong-field
//! data: the fields exist but carry the wrong JS type, which reaches the same
//! wasm-bindgen string shim and used to throw out of the global listeners.
//! The helpers are exercised directly (unit) and through the two listeners
//! mounted on the login route (integration).

#![cfg(all(target_arch = "wasm32", test))]

use js_sys::{Object, Reflect};
use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::KeyboardEvent;

use crate::core::tauri_mock::TauriMock;
use crate::hooks::keyboard_shortcuts::install_reload_guard;
use crate::test_support::{create_wrapper, mount_disposable, shared_cell, wait_until};
use crate::ui_components::Modal;
use crate::utils::{event_bool_field, event_string_field};

wasm_bindgen_test_configure!(run_in_browser);

/// Install an own data property over a read-only DOM getter so the JS shim
/// receives the wrong type instead of the field the constructor declared.
fn override_field(event: &JsValue, field: &str, value: &JsValue) {
    let descriptor = Object::new();
    Reflect::set(&descriptor, &JsValue::from_str("value"), value).unwrap();
    Reflect::set(
        &descriptor,
        &JsValue::from_str("configurable"),
        &JsValue::from_bool(true),
    )
    .unwrap();
    let event_object: &Object = event.unchecked_ref();
    Object::define_property(event_object, &JsValue::from_str(field), &descriptor);
}

fn object_with(fields: &[(&str, JsValue)]) -> Object {
    let object = Object::new();
    for (name, value) in fields {
        Reflect::set(&object, &JsValue::from_str(name), value).unwrap();
    }
    object
}

#[wasm_bindgen_test]
fn string_field_reads_only_strings() {
    let event = object_with(&[
        ("code", JsValue::from_str("KeyR")),
        ("key", JsValue::from_str("Escape")),
        ("number", JsValue::from_f64(42.0)),
        ("boolean", JsValue::from_bool(true)),
        ("nothing", JsValue::NULL),
    ]);
    let event: JsValue = event.into();

    assert_eq!(event_string_field(&event, "code"), "KeyR");
    assert_eq!(event_string_field(&event, "key"), "Escape");
    assert_eq!(event_string_field(&event, "number"), "");
    assert_eq!(event_string_field(&event, "boolean"), "");
    assert_eq!(event_string_field(&event, "nothing"), "");
    assert_eq!(event_string_field(&event, "missing"), "");
}

#[wasm_bindgen_test]
fn bool_field_reads_only_booleans() {
    let event = object_with(&[
        ("ctrlKey", JsValue::from_bool(true)),
        ("metaKey", JsValue::from_bool(false)),
        ("string", JsValue::from_str("true")),
        ("number", JsValue::from_f64(1.0)),
        ("nothing", JsValue::NULL),
    ]);
    let event: JsValue = event.into();

    assert!(event_bool_field(&event, "ctrlKey"));
    assert!(!event_bool_field(&event, "metaKey"));
    assert!(!event_bool_field(&event, "string"));
    assert!(!event_bool_field(&event, "number"));
    assert!(!event_bool_field(&event, "nothing"));
    assert!(!event_bool_field(&event, "missing"));
}

/// Record only exceptions raised by the listeners under test: other browser
/// tests leave their own keydown listeners mounted on `document`.
fn install_error_capture() {
    js_sys::eval(
        "window.__typedKeydownErrors = []; \
         if (!window.__typedKeydownErrorHandler) { \
             window.__typedKeydownErrorHandler = function(e) { \
                 window.__typedKeydownErrors.push(String((e.error && e.error.stack) || e.message || e.error)); \
             }; \
             window.addEventListener('error', window.__typedKeydownErrorHandler); \
         }",
    )
    .expect("install typed-keydown error capture");
}

fn our_listener_errors() -> Vec<String> {
    let window = web_sys::window().expect("window");
    let value = Reflect::get(&window.into(), &JsValue::from_str("__typedKeydownErrors"))
        .expect("read captured errors");
    js_sys::Array::from(&value)
        .iter()
        .filter_map(|value| value.as_string())
        .filter(|message| {
            message.contains("hooks::keyboard_shortcuts::install_reload_guard")
                || message.contains("ui_components::modal::")
        })
        .collect()
}

#[wasm_bindgen_test]
async fn wrong_typed_keydown_fields_do_not_trap_either_login_listener() {
    install_error_capture();
    let wrapper = create_wrapper();
    let _tauri = TauriMock::install(|_, _| JsValue::UNDEFINED);
    js_sys::eval(
        "window.__typedReloadCalls = 0; \
         window.__TAURI__.webview = { \
             getCurrentWebview: function() { \
                 return { reload: function() { window.__typedReloadCalls++; } }; \
             } \
         };",
    )
    .expect("install reload stub");

    let (set_open, get_open) = shared_cell::<RwSignal<bool>>();
    let _mount = mount_disposable(&wrapper, move || {
        let is_open = RwSignal::new(false);
        set_open.set(Some(is_open));
        install_reload_guard();
        view! {
            <input id="typed-email" type="email" />
            <Modal
                is_open=is_open
                title=Signal::derive(|| "Typed repro".to_string())
                test_id="typed-repro-modal"
            >
                "body"
            </Modal>
        }
        .into_any()
    });
    let is_open = get_open.get().expect("captured is_open");
    is_open.set(true);
    tick().await;
    let input = wrapper
        .query_selector("input#typed-email")
        .unwrap()
        .unwrap();

    let init = web_sys::KeyboardEventInit::new();
    init.set_bubbles(true);
    init.set_key("x");
    init.set_code("KeyR");
    init.set_ctrl_key(true);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap();
    let event: JsValue = event.into();
    override_field(&event, "code", &JsValue::from_f64(42.0));
    let wrong_key = JsValue::from(object_with(&[("raw", JsValue::from_str("x"))]));
    override_field(&event, "key", &wrong_key);
    override_field(&event, "ctrlKey", &JsValue::from_str("true"));
    // Prove the fixture really carries wrong types, not the declared strings.
    assert!(
        Reflect::get(&event, &JsValue::from_str("code"))
            .unwrap()
            .as_f64()
            .is_some()
    );

    input.dispatch_event(&event.unchecked_into()).unwrap();
    tick().await;

    assert!(
        our_listener_errors().is_empty(),
        "wrong-typed keydown fields must not throw from either login listener: {:?}",
        our_listener_errors()
    );
    let reload_calls = Reflect::get(
        &web_sys::window().unwrap().into(),
        &JsValue::from_str("__typedReloadCalls"),
    )
    .unwrap()
    .as_f64()
    .unwrap_or_default();
    assert_eq!(reload_calls, 0.0, "a non-string code must not reload");

    let still_open = wait_until(
        || {
            wrapper
                .query_selector("[data-testid=\"typed-repro-modal\"]")
                .unwrap()
                .is_some()
        },
        5,
        25,
    )
    .await;
    assert!(
        still_open,
        "a non-string key must not close the modal like Escape"
    );
}
