#![cfg(all(target_arch = "wasm32", test))]

//! Browser tests for the store-review gate (issue #729, ADR-062).
//!
//! `window.__TAURI__` is stubbed with a controllable `core.invoke` — the
//! native `request_app_review` command cannot run headless, so the stub's
//! resolved boolean models the native verdict. Everything around it (gate,
//! counters, cooldown persistence) is the real code.

use gloo_storage::{LocalStorage, Storage};
use js_sys::{Function, Object, Promise, Reflect};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_test::*;

use crate::core::review_request::{LAST_PROMPT_KEY, SESSIONS_KEY, maybe_request_app_review};

wasm_bindgen_test_configure!(run_in_browser);

const HOUR: u64 = 60 * 60;

/// `maybe_request_app_review` spawns through `leptos::task::spawn_local`,
/// whose executor initializes on mount (see kanji_art_heal_wasm_tests for
/// the same dependency) — mount a throwaway component first.
fn init_leptos_executor() {
    let wrapper = crate::test_support::create_wrapper();
    crate::test_support::mount_to_wrapper(&wrapper, move || {
        view! { <div data-testid="review-request-test-mount"></div> }.into_any()
    });
}

/// Installs `window.__TAURI__.core.invoke` returning
/// `Promise::resolve(resolve_value)`; the returned closure must be kept
/// alive for the duration of the test (dropping it invalidates the
/// function the stub points at).
fn install_invoke_stub(resolve_value: bool) -> Closure<dyn Fn(JsValue, JsValue) -> JsValue> {
    let closure = Closure::new(move |_command: JsValue, _args: JsValue| {
        JsValue::from(Promise::resolve(&JsValue::from_bool(resolve_value)))
    });
    let window = web_sys::window().expect("browser test window");
    let tauri = Object::new();
    let core = Object::new();
    let invoke = closure.as_ref().unchecked_ref::<Function>();
    Reflect::set(&core, &JsValue::from_str("invoke"), invoke).expect("set invoke");
    Reflect::set(&tauri, &JsValue::from_str("core"), &core).expect("set core");
    Reflect::set(&window, &JsValue::from_str("__TAURI__"), &tauri).expect("set __TAURI__");
    closure
}

fn remove_invoke_stub() {
    let window = web_sys::window().expect("browser test window");
    Reflect::delete_property(&window, &JsValue::from_str("__TAURI__")).expect("delete __TAURI__");
}

fn reset_gate_storage() {
    LocalStorage::delete(LAST_PROMPT_KEY);
    LocalStorage::delete(SESSIONS_KEY);
}

/// Lets the detached `spawn_local` task (invoke + persistence) run to
/// completion before the assertions read localStorage.
async fn flush_spawned_task() {
    gloo_timers::future::TimeoutFuture::new(100).await;
}

#[wasm_bindgen_test]
async fn zero_cards_neither_counts_nor_prompts() {
    init_leptos_executor();
    reset_gate_storage();
    remove_invoke_stub();

    maybe_request_app_review(0);

    assert!(
        LocalStorage::get::<u64>(SESSIONS_KEY).is_err(),
        "zero-card session must not be counted"
    );
    assert!(
        LocalStorage::get::<u64>(LAST_PROMPT_KEY).is_err(),
        "zero-card session must not trigger a prompt"
    );
}

#[wasm_bindgen_test]
async fn gate_blocks_invoke_while_cooldown_active() {
    init_leptos_executor();
    reset_gate_storage();
    LocalStorage::set(SESSIONS_KEY, 2_u64).expect("seed sessions");
    let now = js_sys::Date::now() as u64 / 1000;
    LocalStorage::set(LAST_PROMPT_KEY, now - HOUR).expect("seed last prompt");

    let _stub = install_invoke_stub(true);
    maybe_request_app_review(3);
    flush_spawned_task().await;

    assert_eq!(
        LocalStorage::get::<u64>(SESSIONS_KEY).ok(),
        Some(3),
        "the session itself is counted even while the gate blocks"
    );
    assert_eq!(
        LocalStorage::get::<u64>(LAST_PROMPT_KEY).ok(),
        Some(now - HOUR),
        "cooldown must not be rewritten"
    );
}

#[wasm_bindgen_test]
async fn native_true_records_prompt_timestamp() {
    init_leptos_executor();
    reset_gate_storage();
    LocalStorage::set(SESSIONS_KEY, 3_u64).expect("seed sessions");

    let stub = install_invoke_stub(true);
    maybe_request_app_review(3);
    flush_spawned_task().await;
    drop(stub);

    assert_eq!(LocalStorage::get::<u64>(SESSIONS_KEY).ok(), Some(4));
    assert!(
        LocalStorage::get::<u64>(LAST_PROMPT_KEY).is_ok(),
        "a requested prompt must burn the cooldown"
    );
}

#[wasm_bindgen_test]
async fn unsupported_platform_keeps_cooldown_intact() {
    init_leptos_executor();
    reset_gate_storage();
    LocalStorage::set(SESSIONS_KEY, 3_u64).expect("seed sessions");

    let stub = install_invoke_stub(false);
    maybe_request_app_review(3);
    flush_spawned_task().await;
    drop(stub);

    assert_eq!(LocalStorage::get::<u64>(SESSIONS_KEY).ok(), Some(4));
    assert!(
        LocalStorage::get::<u64>(LAST_PROMPT_KEY).is_err(),
        "a platform without the review API must not burn the cooldown"
    );
}
