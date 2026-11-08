//! WASM tests for the `device-ai-apis` JS bridge contracts.
//!
//! Two contract groups live here:
//!
//! - `vision_recognize_text` payload: the plugin splits language selection by
//!   platform (iOS/macOS/Windows read `options.languages`, Android ML Kit
//!   reads `options.script` and silently defaults to the Latin recognizer
//!   when it is absent — which broke Japanese OCR on Android). These tests
//!   pin the payload so the cross-platform fields cannot regress.
//! - `get_capabilities` single-flight + cache (#570): concurrent first
//!   callers must share one query and all receive the real result — the old
//!   `CAPABILITIES_LOADING` shortcut handed the loser a synthetic
//!   "unavailable", and one-shot callers (live-recorder mount effect) never
//!   re-checked, permanently hiding the macOS record button.
//!
//! Every test is order-independent: module state (capabilities cache,
//! in-flight query slot) and the fake `window.__TAURI__` bridge are reset at
//! the start and the end of each test, with a per-test invoke counter.

#![cfg(all(target_arch = "wasm32", test))]

use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
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

// =============================================================================
// Capabilities: single-flight query + cache (#570)
// =============================================================================

use crate::core::device_ai::{self, Feature};

/// How long a test waits for spawned tasks, in 10 ms ticks.
const COMPLETION_TICKS: u32 = 300;
/// Fake invoke resolution delay — long enough that concurrently spawned
/// callers are guaranteed to race for the same in-flight query.
const FAKE_INVOKE_DELAY_MS: i32 = 25;

/// Resets every piece of module state a capabilities test can observe:
/// the cache, the in-flight query slot and any fake bridge from a previous
/// test. Called at the start and the end of each test so execution order
/// cannot couple them.
fn reset_module_state() {
    remove_fake_tauri();
    super::invoke::reset_in_flight_query();
    super::reset_cached_capabilities();
}

/// Installs a fake `window.__TAURI__.core.invoke` that counts calls and
/// resolves with `payload` after [`FAKE_INVOKE_DELAY_MS`].
fn install_fake_tauri(payload: &JsValue) {
    let window = web_sys::window().expect("test browser window");
    let invoke_body = format!(
        "window.__fakeInvokeCount = (window.__fakeInvokeCount || 0) + 1; \
         return new Promise(function (resolve) {{ \
             setTimeout(function () {{ resolve(window.__fakeDeviceAiCaps); }}, \
                 {FAKE_INVOKE_DELAY_MS}); \
         }});"
    );
    let invoke_fn = js_sys::Function::new_with_args("command, args", &invoke_body);

    let core = js_sys::Object::new();
    js_sys::Reflect::set(&core, &JsValue::from_str("invoke"), &invoke_fn.into())
        .expect("set core.invoke");
    let tauri = js_sys::Object::new();
    js_sys::Reflect::set(&tauri, &JsValue::from_str("core"), &core.into())
        .expect("set __TAURI__.core");

    js_sys::Reflect::set(&window, &JsValue::from_str("__fakeDeviceAiCaps"), payload)
        .expect("set fake payload");
    js_sys::Reflect::set(
        &window,
        &JsValue::from_str("__fakeInvokeCount"),
        &JsValue::from_f64(0.0),
    )
    .expect("reset invoke counter");
    js_sys::Reflect::set(&window, &JsValue::from_str("__TAURI__"), &tauri.into())
        .expect("install fake __TAURI__");
}

/// Installs a fake `window.__TAURI__.core.invoke` whose promise always
/// rejects — models a plugin/bridge failure so retry semantics can be pinned.
fn install_failing_tauri() {
    let window = web_sys::window().expect("test browser window");
    let invoke_fn = js_sys::Function::new_with_args(
        "command, args",
        "window.__fakeInvokeCount = (window.__fakeInvokeCount || 0) + 1; \
         return Promise.reject(new Error('fake bridge down'));",
    );

    let core = js_sys::Object::new();
    js_sys::Reflect::set(&core, &JsValue::from_str("invoke"), &invoke_fn.into())
        .expect("set core.invoke");
    let tauri = js_sys::Object::new();
    js_sys::Reflect::set(&tauri, &JsValue::from_str("core"), &core.into())
        .expect("set __TAURI__.core");

    js_sys::Reflect::set(
        &window,
        &JsValue::from_str("__fakeInvokeCount"),
        &JsValue::from_f64(0.0),
    )
    .expect("reset invoke counter");
    js_sys::Reflect::set(&window, &JsValue::from_str("__TAURI__"), &tauri.into())
        .expect("install failing __TAURI__");
}

/// Installs a fake `window.__TAURI__.core.invoke` that throws synchronously —
/// models a bridge that cannot even start the call (`invoke_promise` returns
/// `Err`), exercising the already-rejected-promise path of the shared slot.
fn install_throwing_tauri() {
    let window = web_sys::window().expect("test browser window");
    let invoke_fn = js_sys::Function::new_with_args(
        "command, args",
        "window.__fakeInvokeCount = (window.__fakeInvokeCount || 0) + 1; \
         throw new Error('sync bridge failure');",
    );

    let core = js_sys::Object::new();
    js_sys::Reflect::set(&core, &JsValue::from_str("invoke"), &invoke_fn.into())
        .expect("set core.invoke");
    let tauri = js_sys::Object::new();
    js_sys::Reflect::set(&tauri, &JsValue::from_str("core"), &core.into())
        .expect("set __TAURI__.core");

    js_sys::Reflect::set(
        &window,
        &JsValue::from_str("__fakeInvokeCount"),
        &JsValue::from_f64(0.0),
    )
    .expect("reset invoke counter");
    js_sys::Reflect::set(&window, &JsValue::from_str("__TAURI__"), &tauri.into())
        .expect("install throwing __TAURI__");
}

/// Removes the fake bridge and its globals.
fn remove_fake_tauri() {
    let window = match web_sys::window() {
        Some(window) => window,
        None => return,
    };
    for key in ["__TAURI__", "__fakeDeviceAiCaps", "__fakeInvokeCount"] {
        let _ = js_sys::Reflect::delete_property(&window, &JsValue::from_str(key));
    }
}

/// Reads the fake invoke call counter installed by [`install_fake_tauri`].
fn invoke_count() -> f64 {
    let window = web_sys::window().expect("test browser window");
    js_sys::Reflect::get(&window, &JsValue::from_str("__fakeInvokeCount"))
        .expect("read invoke counter")
        .as_f64()
        .expect("invoke counter is a number")
}

/// Builds the plugin's capabilities payload (camelCase wire format) with
/// speech recognition set to `speech_available`; every other feature stays
/// absent, which the frontend contract decodes as unavailable.
fn capabilities_payload(speech_available: bool) -> JsValue {
    let speech = js_sys::Object::new();
    js_sys::Reflect::set(
        &speech,
        &JsValue::from_str("available"),
        &JsValue::from_bool(speech_available),
    )
    .expect("set speechRecognition.available");

    let caps = js_sys::Object::new();
    js_sys::Reflect::set(
        &caps,
        &JsValue::from_str("speechRecognition"),
        &speech.into(),
    )
    .expect("set speechRecognition");
    caps.into()
}

/// Waits (bounded) until `remaining` drops to zero; panics on timeout so a
/// hung task fails the test instead of stalling the suite.
async fn wait_for_tasks(remaining: &Rc<Cell<u32>>) {
    for _ in 0..COMPLETION_TICKS {
        if remaining.get() == 0 {
            return;
        }
        gloo_timers::future::TimeoutFuture::new(10).await;
    }
    panic!("concurrent tasks did not finish within the wait budget");
}

#[wasm_bindgen_test]
async fn concurrent_first_callers_share_one_query_and_all_get_the_real_result() {
    reset_module_state();
    install_fake_tauri(&capabilities_payload(true));

    let remaining = Rc::new(Cell::new(2_u32));
    let results: Rc<std::cell::RefCell<Vec<bool>>> = Rc::new(std::cell::RefCell::new(Vec::new()));
    for _ in 0..2 {
        let remaining = remaining.clone();
        let results = results.clone();
        spawn_local(async move {
            let available = device_ai::available(Feature::SpeechRecognition).await;
            results.borrow_mut().push(available);
            remaining.set(remaining.get() - 1);
        });
    }
    wait_for_tasks(&remaining).await;

    let results = results.borrow();
    assert_eq!(
        results.len(),
        2,
        "both concurrent callers must have completed"
    );
    assert!(
        results.iter().all(|&available| available),
        "no caller may receive the synthetic unavailable: {results:?}"
    );
    assert_eq!(
        invoke_count(),
        1.0,
        "concurrent first callers share one plugin query"
    );

    reset_module_state();
}

#[wasm_bindgen_test]
async fn repeated_call_hits_the_cache_without_a_second_invoke() {
    reset_module_state();
    install_fake_tauri(&capabilities_payload(true));

    assert!(
        device_ai::available(Feature::SpeechRecognition).await,
        "first call resolves the query"
    );
    assert!(
        device_ai::available(Feature::SpeechRecognition).await,
        "second call reads the warm cache"
    );
    assert_eq!(
        invoke_count(),
        1.0,
        "cache hit must not re-invoke the plugin"
    );

    reset_module_state();
}

#[wasm_bindgen_test]
async fn an_unavailable_payload_is_cached_like_any_successful_query() {
    reset_module_state();
    install_fake_tauri(&capabilities_payload(false));

    assert!(
        !device_ai::available(Feature::SpeechRecognition).await,
        "unavailable capabilities resolve false"
    );
    assert!(
        !device_ai::available(Feature::SpeechRecognition).await,
        "the cached honest unavailable still serves false"
    );
    assert_eq!(
        invoke_count(),
        1.0,
        "an all-unavailable payload is a successful query and must be cached"
    );

    reset_module_state();
}

#[wasm_bindgen_test]
async fn a_failed_query_is_not_cached_so_the_next_call_retries() {
    reset_module_state();
    install_failing_tauri();

    assert!(
        !device_ai::available(Feature::SpeechRecognition).await,
        "a failed query routes to the fallback"
    );
    assert!(
        !device_ai::available(Feature::SpeechRecognition).await,
        "the retry also fails while the bridge stays down"
    );
    assert_eq!(
        invoke_count(),
        2.0,
        "failures are not cached — each call retries the query"
    );

    reset_module_state();
}

#[wasm_bindgen_test]
async fn a_synchronously_throwing_invoke_is_not_cached_so_the_next_call_retries() {
    reset_module_state();
    install_throwing_tauri();

    assert!(
        !device_ai::available(Feature::SpeechRecognition).await,
        "a bridge that cannot even start the call routes to the fallback"
    );
    assert!(
        !device_ai::available(Feature::SpeechRecognition).await,
        "the retry also fails while the bridge keeps throwing"
    );
    assert_eq!(
        invoke_count(),
        2.0,
        "a synchronous invoke failure must not be cached — each call retries"
    );

    reset_module_state();
}

#[wasm_bindgen_test]
async fn three_concurrent_first_callers_share_one_query_and_all_get_the_real_result() {
    reset_module_state();
    install_fake_tauri(&capabilities_payload(true));

    let remaining = Rc::new(Cell::new(3_u32));
    let results: Rc<std::cell::RefCell<Vec<bool>>> = Rc::new(std::cell::RefCell::new(Vec::new()));
    for _ in 0..3 {
        let remaining = remaining.clone();
        let results = results.clone();
        spawn_local(async move {
            let available = device_ai::available(Feature::SpeechRecognition).await;
            results.borrow_mut().push(available);
            remaining.set(remaining.get() - 1);
        });
    }
    wait_for_tasks(&remaining).await;

    let results = results.borrow();
    assert_eq!(
        results.len(),
        3,
        "all three concurrent callers must have completed"
    );
    assert!(
        results.iter().all(|&available| available),
        "no caller may receive the synthetic unavailable: {results:?}"
    );
    assert_eq!(
        invoke_count(),
        1.0,
        "the shared query slot must survive until the query settles"
    );

    reset_module_state();
}
