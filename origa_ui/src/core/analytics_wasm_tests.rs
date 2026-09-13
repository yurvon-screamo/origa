//! WASM tests for [`crate::core::analytics::inject_umami`] — verify the ON
//! build path: with the default `UMAMI_WEBSITE_ID` (the wasm-test job does
//! not set `UMAMI_DISABLED`), injection must append a correctly configured
//! tracker script to `<head>`. The e2e guard (smoke.feature) covers the OFF
//! path instead; these two complement so neither regression is silent.

use crate::core::analytics::{UMAMI_SCRIPT_SRC, inject_umami};
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
