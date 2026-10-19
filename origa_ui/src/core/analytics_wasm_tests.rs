//! WASM tests for [`crate::core::analytics::inject_umami`] — verify the ON
//! build path: with the default `UMAMI_WEBSITE_ID` (the wasm-test job does
//! not set `UMAMI_DISABLED`), injection must append a correctly configured
//! tracker script to `<head>`. The e2e guard (smoke.feature) covers the OFF
//! path instead; these two complement so neither regression is silent.

use crate::core::analytics::{UMAMI_SCRIPT_SRC, inject_umami, track_event};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

/// Remove trackers left by a previous test, inject a fresh one, return it.
/// Keeps the tests isolated — `inject_umami` itself is called exactly once
/// per app boot, so it has no built-in idempotency guard.
fn fresh_tracker_script() -> web_sys::Element {
    let document = web_sys::window()
        .expect("window in browser test")
        .document()
        .expect("document");
    let head = document.head().expect("test page has a <head>");
    while let Some(stale) = head
        .query_selector("script[data-website-id]")
        .expect("querySelector on <head>")
    {
        stale.remove();
    }
    inject_umami();
    head.query_selector("script[data-website-id]")
        .expect("querySelector on <head>")
        .expect("tracker script appended to <head>")
}

#[wasm_bindgen_test]
fn inject_umami_appends_tracker_script_to_head() {
    let script = fresh_tracker_script();

    assert_eq!(
        script.get_attribute("src").as_deref(),
        Some(UMAMI_SCRIPT_SRC)
    );
    assert_eq!(
        script.get_attribute("data-website-id").as_deref(),
        Some(env!("UMAMI_WEBSITE_ID"))
    );
}

#[wasm_bindgen_test]
fn inject_umami_sets_domain_allowlist_and_do_not_track() {
    let script = fresh_tracker_script();

    let domains = script
        .get_attribute("data-domains")
        .expect("data-domains attribute present");
    assert!(domains.contains("app.origa.uwuwu.net"), "got: {domains}");
    assert!(domains.contains("tauri.localhost"), "got: {domains}");
    assert!(domains.contains("localhost"), "got: {domains}");
    assert_eq!(
        script.get_attribute("data-do-not-track").as_deref(),
        Some("true")
    );
}

// ═══════════════════════════════════════════════════════════════════════
// track_event (programmatic events, ADR-054 §5)
// ═══════════════════════════════════════════════════════════════════════

/// Remove the `umami` global and every queue/flusher artifact a previous
/// test left on the shared test page. The real tracker (if the page ever
/// fetched one) would overwrite `umami` on its own, so deletion cannot
/// break it here.
fn reset_umami_global(window: &web_sys::Window) {
    for global in [
        "umami",
        "__umami_test_calls",
        "__umami_pending",
        "__umami_enqueue",
        "__umami_flush_timer",
        "__umami_flush_tries",
    ] {
        let _ = js_sys::Reflect::delete_property(window, &wasm_bindgen::JsValue::from_str(global));
    }
}

#[wasm_bindgen_test]
fn track_event_calls_umami_track_with_this_name_and_props() {
    let window = web_sys::window().expect("window in browser test");
    reset_umami_global(&window);

    // Spy: a `track` method recording name, data and the receiver, so the
    // contract `umami.track(name, data)` with `this` = the umami object is
    // asserted end-to-end (a detached receiver is exactly the silent-drop
    // failure mode this guard exists for).
    let track_fn = js_sys::Function::new_with_args(
        "name, data",
        "window.__umami_test_calls.push({ receiver: this, name: name, data: data });",
    );
    let umami = js_sys::Object::new();
    js_sys::Reflect::set(&umami, &wasm_bindgen::JsValue::from_str("track"), &track_fn)
        .expect("set track on umami");
    js_sys::Reflect::set(&window, &wasm_bindgen::JsValue::from_str("umami"), &umami)
        .expect("set umami global");
    js_sys::Reflect::set(
        &window,
        &wasm_bindgen::JsValue::from_str("__umami_test_calls"),
        &js_sys::Array::new().into(),
    )
    .expect("set call log");

    track_event(
        "onboarding_intro_start",
        &[("locale", "ru"), ("device", "desktop")],
    );

    let calls = js_sys::Reflect::get(
        &window,
        &wasm_bindgen::JsValue::from_str("__umami_test_calls"),
    )
    .expect("call log present");
    let calls: js_sys::Array = calls.dyn_into().expect("call log is an array");
    assert_eq!(calls.length(), 1, "exactly one track call expected");

    let call = calls.get(0);
    let receiver = js_sys::Reflect::get(&call, &wasm_bindgen::JsValue::from_str("receiver"))
        .expect("receiver recorded");
    // JsValue's `==` is `Object.is` — identity for the same object, exactly
    // what the receiver assertion needs.
    let receiver_is_umami = receiver == wasm_bindgen::JsValue::from(umami.clone());
    assert!(
        receiver_is_umami,
        "track must be invoked with the umami object as `this`"
    );

    let name = js_sys::Reflect::get(&call, &wasm_bindgen::JsValue::from_str("name"))
        .expect("name recorded")
        .as_string()
        .expect("name is a string");
    assert_eq!(name, "onboarding_intro_start");

    let data = js_sys::Reflect::get(&call, &wasm_bindgen::JsValue::from_str("data"))
        .expect("data recorded");
    let locale = js_sys::Reflect::get(&data, &wasm_bindgen::JsValue::from_str("locale"))
        .expect("locale prop")
        .as_string()
        .expect("locale is a string");
    assert_eq!(locale, "ru");
    let device = js_sys::Reflect::get(&data, &wasm_bindgen::JsValue::from_str("device"))
        .expect("device prop")
        .as_string()
        .expect("device is a string");
    assert_eq!(device, "desktop");

    reset_umami_global(&window);
}

#[wasm_bindgen_test]
fn track_event_without_tracker_is_silent_no_op() {
    let window = web_sys::window().expect("window in browser test");
    reset_umami_global(&window);

    // No `umami` global (tracker not loaded / compiled out): the call must
    // neither panic nor throw — the event is queued (and eventually
    // discarded by the flusher's give-up bound).
    track_event("orphan_event", &[("locale", "en")]);

    reset_umami_global(&window);
}

#[wasm_bindgen_test]
async fn track_event_queues_until_tracker_appears_and_flushes_in_order() {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().expect("window in browser test");
    reset_umami_global(&window);

    // Cold start: the tracker is absent, both events must be queued.
    track_event("queued_first", &[("locale", "ru")]);
    track_event("queued_second", &[("device", "desktop")]);

    // The tracker "loads": install the spy AFTER the events were queued.
    let track_fn = js_sys::Function::new_with_args(
        "name, data",
        "window.__umami_test_calls.push({ receiver: this, name: name, data: data });",
    );
    let umami = js_sys::Object::new();
    js_sys::Reflect::set(&umami, &wasm_bindgen::JsValue::from_str("track"), &track_fn)
        .expect("set track on umami");
    js_sys::Reflect::set(&window, &wasm_bindgen::JsValue::from_str("umami"), &umami)
        .expect("set umami global");
    js_sys::Reflect::set(
        &window,
        &wasm_bindgen::JsValue::from_str("__umami_test_calls"),
        &js_sys::Array::new().into(),
    )
    .expect("set call log");

    // Wait past one flusher tick (200 ms interval): the queued events must
    // drain into the spy in FIFO order.
    let sleep = js_sys::Promise::new(&mut |resolve, _| {
        let callback = wasm_bindgen::closure::Closure::once(move |_: wasm_bindgen::JsValue| {
            let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
        });
        let _ = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                500,
            )
            .expect("setTimeout");
        callback.forget();
    });
    let _ = wasm_bindgen_futures::JsFuture::from(sleep).await;

    let calls = js_sys::Reflect::get(
        &window,
        &wasm_bindgen::JsValue::from_str("__umami_test_calls"),
    )
    .expect("call log present");
    let calls: js_sys::Array = calls.dyn_into().expect("call log is an array");
    assert_eq!(calls.length(), 2, "both queued events must flush");

    let first_name = js_sys::Reflect::get(&calls.get(0), &wasm_bindgen::JsValue::from_str("name"))
        .expect("first name")
        .as_string()
        .expect("string");
    assert_eq!(first_name, "queued_first", "FIFO order required");
    let second_name = js_sys::Reflect::get(&calls.get(1), &wasm_bindgen::JsValue::from_str("name"))
        .expect("second name")
        .as_string()
        .expect("string");
    assert_eq!(second_name, "queued_second", "FIFO order required");

    let first_data = js_sys::Reflect::get(&calls.get(0), &wasm_bindgen::JsValue::from_str("data"))
        .expect("first data");
    let locale = js_sys::Reflect::get(&first_data, &wasm_bindgen::JsValue::from_str("locale"))
        .expect("locale prop")
        .as_string()
        .expect("string");
    assert_eq!(locale, "ru");

    reset_umami_global(&window);
}
