//! Test-only doubles for the Tauri JS bridge and the runtime platform.
//!
//! Behaviour tests for the device-ai routing need a controllable plugin:
//! a scriptable `window.__TAURI__.core.invoke` (resolve capabilities /
//! reject specific commands) and an overridable `navigator.userAgent` (the
//! platform gate reads `platform_name()`, which derives "android"/"web"
//! from the UA string). Both restore the page's original state on drop.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use leptos::wasm_bindgen::prelude::Closure;
use web_sys::window;

/// Installs a scripted `window.__TAURI__` whose `core.invoke` delegates to
/// `route`. The route receives the command name and returns the JS value to
/// resolve the invocation with — a plain value resolves, a rejected
/// `Promise` (see [`rejected_promise`]) simulates a plugin failure.
pub struct TauriMock {
    _closure: Closure<dyn Fn(JsValue, JsValue) -> JsValue>,
}

impl TauriMock {
    pub fn install(route: impl Fn(JsValue, JsValue) -> JsValue + 'static) -> Self {
        let closure = Closure::new(move |command: JsValue, args: JsValue| {
            let result = route(command, args);
            // The bridge contract (`core::tauri::invoke_promise`) dyn_casts
            // the invoke result to a Promise — wrap plain values (e.g. a
            // capabilities object) into resolved promises; existing
            // promises pass through untouched.
            if result.has_type::<js_sys::Promise>() {
                result
            } else {
                js_sys::Promise::resolve(&result).into()
            }
        });
        let window = window().expect("test browser window");
        let invoke_fn = closure.as_ref().unchecked_ref::<js_sys::Function>().into();

        let core = js_sys::Object::new();
        js_sys::Reflect::set(&core, &JsValue::from_str("invoke"), &invoke_fn)
            .expect("set core.invoke");

        let tauri = js_sys::Object::new();
        js_sys::Reflect::set(&tauri, &JsValue::from_str("core"), &core).expect("set core");

        js_sys::Reflect::set(&window, &JsValue::from_str("__TAURI__"), &tauri)
            .expect("set __TAURI__");
        Self { _closure: closure }
    }
}

impl Drop for TauriMock {
    fn drop(&mut self) {
        let window = window().expect("test browser window");
        let _ = js_sys::Reflect::delete_property(&window, &JsValue::from_str("__TAURI__"));
    }
}

/// A `Promise` rejected with an Error object carrying `code` and `message`
/// — the shape the plugin's IPC failures take on the JS bridge, and the
/// format `describe_rejection` turns back into `"[CODE] message"` strings
/// the escalation points log.
pub fn rejected_with(code: &str, message: &str) -> JsValue {
    let error = js_sys::Error::new(message);
    js_sys::Reflect::set(&error, &JsValue::from_str("code"), &JsValue::from_str(code))
        .expect("set error.code");
    let mut reject_with_error = move |_resolve: js_sys::Function, reject: js_sys::Function| {
        let _ = reject.call1(&JsValue::UNDEFINED, &error);
    };
    js_sys::Promise::new(&mut reject_with_error).into()
}

/// Capabilities payload with every feature available — resolves the
/// `get_capabilities` command so the routing reaches the invoke-fail paths
/// (which are only reachable after a caps=available verdict).
pub fn all_available_caps() -> JsValue {
    let feature = |name: &str| {
        let status = js_sys::Object::new();
        js_sys::Reflect::set(&status, &JsValue::from_str("available"), &JsValue::TRUE)
            .expect("set available");
        (JsValue::from_str(name), JsValue::from(status))
    };
    let caps = js_sys::Object::new();
    for (key, value) in [
        feature("speechRecognition"),
        feature("speechSynthesis"),
        feature("textRecognition"),
    ] {
        js_sys::Reflect::set(&caps, &key, &value).expect("set caps feature");
    }
    caps.into()
}

/// Overrides `navigator.userAgent` for the duration of the guard's lifetime
/// (RAII restore on drop).
///
/// `platform_name()` reads the UA before the Tauri check, so an Android UA
/// alone pins the platform to "android" even in a plain browser page — the
/// same precedence the device gate relies on.
pub struct UserAgentOverride {
    original: Option<JsValue>,
}

impl UserAgentOverride {
    pub fn install(ua: &str) -> Self {
        let window = window().expect("test browser window");
        let navigator = window.navigator();
        let prototype = js_sys::Object::get_prototype_of(&navigator);
        let key = JsValue::from_str("userAgent");

        let original = js_sys::Object::get_own_property_descriptor(&prototype, &key);
        let had_own = !original.is_undefined() && !original.is_null();

        let descriptor = js_sys::Object::new();
        js_sys::Reflect::set(
            &descriptor,
            &JsValue::from_str("value"),
            &JsValue::from_str(ua),
        )
        .expect("set value");
        js_sys::Reflect::set(&descriptor, &JsValue::from_str("writable"), &JsValue::TRUE)
            .expect("set writable");
        js_sys::Reflect::set(
            &descriptor,
            &JsValue::from_str("enumerable"),
            &JsValue::TRUE,
        )
        .expect("set enumerable");
        js_sys::Reflect::set(
            &descriptor,
            &JsValue::from_str("configurable"),
            &JsValue::TRUE,
        )
        .expect("set configurable");

        js_sys::Object::define_property(&prototype, &key, &descriptor);

        Self {
            original: had_own.then_some(original),
        }
    }
}

impl Drop for UserAgentOverride {
    fn drop(&mut self) {
        let window = window().expect("test browser window");
        let navigator = window.navigator();
        let prototype = js_sys::Object::get_prototype_of(&navigator);
        let key = JsValue::from_str("userAgent");
        match self.original.take() {
            // A restored descriptor is a data descriptor (a plain object);
            // a failure here would surface as a following-test UA leak, so
            // it panics loudly instead of swallowing.
            Some(descriptor) => {
                let descriptor = descriptor
                    .dyn_into::<js_sys::Object>()
                    .expect("userAgent descriptor is an object");
                js_sys::Object::define_property(&prototype, &key, &descriptor);
            },
            None => {
                let _ = js_sys::Reflect::delete_property(&prototype, &key);
            },
        }
    }
}
