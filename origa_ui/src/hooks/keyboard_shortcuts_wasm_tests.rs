//! Browser-level regression tests for the WASM keydown boundary (uwuwu#1702,
//! RUST-7). Two global listeners are mounted on the login route: the
//! `window` reload guard reads `KeyboardEvent.code`, and the `document`
//! listener inside `Modal` reads `KeyboardEvent.key`. A `keydown` whose
//! event object has no string fields reaches both through bubbling from the
//! focused email input; the wasm-bindgen string shims turn the missing
//! field into `TypeError: ... reading 'length'` in `passStringToWasm0`.
//!
//! The reproduction dispatches on `input#email` (not directly on `window`)
//! so a green test proves both listeners survive the real delivery path.

#![cfg(all(target_arch = "wasm32", test))]

use js_sys::Reflect;
use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use web_sys::{KeyboardEvent, KeyboardEventInit};

use crate::core::tauri_mock::TauriMock;
use crate::hooks::keyboard_shortcuts::install_reload_guard;
use crate::test_support::{create_wrapper, mount_disposable, shared_cell, wait_until};
use crate::ui_components::Modal;

wasm_bindgen_test_configure!(run_in_browser);

/// Records uncaught listener exceptions the same way the Sentry global
/// `onerror` handler does. Installed once; each test resets the array.
fn install_error_capture() {
    js_sys::eval(
        "window.__keydownErrors = []; \
         if (!window.__keydownErrorHandler) { \
             window.__keydownErrorHandler = function(e) { \
                 window.__keydownErrors.push(String(e.message || e.error)); \
             }; \
             window.addEventListener('error', window.__keydownErrorHandler); \
         }",
    )
    .expect("install error capture");
}

fn captured_errors() -> Vec<String> {
    let window = web_sys::window().expect("window");
    let value = Reflect::get(&window.into(), &JsValue::from_str("__keydownErrors"))
        .expect("read captured errors");
    js_sys::Array::from(&value)
        .iter()
        .map(|value| value.as_string().unwrap_or_default())
        .collect()
}

/// Stub the native WebView reload so the guard's call is observable and no
/// real navigation happens in the test browser.
fn install_reload_stub() {
    js_sys::eval(
        "window.__reloadCalls = 0; \
         window.__TAURI__.webview = { \
             getCurrentWebview: function() { \
                 return { reload: function() { window.__reloadCalls++; } }; \
             } \
         };",
    )
    .expect("install reload stub");
}

fn reload_calls() -> u32 {
    let window = web_sys::window().expect("window");
    Reflect::get(&window.into(), &JsValue::from_str("__reloadCalls"))
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or_default() as u32
}

fn dispatch_bare_keydown() {
    js_sys::eval(
        "(function() { \
             var e = new Event('keydown', { bubbles: true }); \
             var inputs = document.querySelectorAll('input#email'); \
             return inputs[inputs.length - 1].dispatchEvent(e); \
         })();",
    )
    .expect("dispatch bare keydown");
}

fn dispatch_key(input: &web_sys::Element, key: &str, code: &str, ctrl: bool) {
    let init = KeyboardEventInit::new();
    init.set_key(key);
    init.set_code(code);
    init.set_ctrl_key(ctrl);
    init.set_bubbles(true);
    let event =
        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).expect("keyboard event");
    input
        .dispatch_event(&event)
        .expect("dispatch keyboard event");
}

/// Mount both production listeners with a closed modal — the exact state of
/// route `/` while the user types into the email field.
fn mount_login_listeners(
    wrapper: &web_sys::Element,
) -> (RwSignal<bool>, web_sys::Element, impl Sized) {
    let (set_open, get_open) = shared_cell::<RwSignal<bool>>();
    let mount = mount_disposable(wrapper, move || {
        let is_open = RwSignal::new(false);
        set_open.set(Some(is_open));
        install_reload_guard();
        view! {
            <input id="email" type="email" />
            <Modal
                is_open=is_open
                title=Signal::derive(|| "Repro".to_string())
                test_id="repro-modal"
            >
                "body"
            </Modal>
        }
        .into_any()
    });
    let is_open = get_open.get().expect("captured is_open");
    let input = wrapper.query_selector("input#email").unwrap().unwrap();
    (is_open, input, mount)
}

#[wasm_bindgen_test]
async fn keydown_without_string_fields_reaches_both_listeners_without_type_error() {
    install_error_capture();
    let wrapper = create_wrapper();
    let _tauri = TauriMock::install(|_, _| JsValue::UNDEFINED);
    let (_is_open, _input, _mount) = mount_login_listeners(&wrapper);
    tick().await;

    dispatch_bare_keydown();

    let errors = captured_errors();
    assert!(
        errors.is_empty(),
        "a bubbled keydown without `code`/`key` must not throw from either global listener, got {errors:?}"
    );
}

#[wasm_bindgen_test]
async fn reload_accelerator_still_uses_the_native_webview_reload() {
    install_error_capture();
    let wrapper = create_wrapper();
    let _tauri = TauriMock::install(|_, _| JsValue::UNDEFINED);
    install_reload_stub();
    let (_is_open, input, _mount) = mount_login_listeners(&wrapper);
    tick().await;

    dispatch_key(&input, "r", "KeyR", true);

    assert_eq!(
        reload_calls(),
        1,
        "Ctrl+R with a valid `code` must stay intercepted (origa PR #222)"
    );
    assert!(captured_errors().is_empty(), "no listener may throw");
}

#[wasm_bindgen_test]
async fn escape_with_a_valid_key_still_closes_the_modal() {
    install_error_capture();
    let wrapper = create_wrapper();
    let _tauri = TauriMock::install(|_, _| JsValue::UNDEFINED);
    let (is_open, input, _mount) = mount_login_listeners(&wrapper);
    is_open.set(true);
    tick().await;
    assert!(
        wrapper
            .query_selector("[data-testid=\"repro-modal\"]")
            .unwrap()
            .is_some(),
        "modal must be open before Escape"
    );

    dispatch_key(&input, "Escape", "Escape", false);

    let closed = wait_until(
        || {
            wrapper
                .query_selector("[data-testid=\"repro-modal\"]")
                .unwrap()
                .is_none()
        },
        20,
        25,
    )
    .await;
    assert!(closed, "Escape with a valid `key` must close the modal");
    assert!(captured_errors().is_empty(), "no listener may throw");
}
