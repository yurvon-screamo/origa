//! WASM tests for the capabilities-failure Sentry escalation gate.
//!
//! The gate (task 20261104051142-inn7qka): a capabilities query failure on
//! a platform that expects the native path (android/ios/macos) must surface
//! as an `error!` event (Sentry-visible), while on plugin-less platforms it
//! stays `warn!` (a breadcrumb — fallback routing is the normal path there).
//! A quiet downgrade back to a single `warn!` would restore the original
//! invisibility bug, so the event level itself is the assertion target.

#![cfg(all(target_arch = "wasm32", test))]

use wasm_bindgen_test::*;

use super::{Feature, available, reset_cached_capabilities};
use crate::core::log_capture;
use crate::core::tauri_mock::{TauriMock, UserAgentOverride, all_available_caps, rejected_with};
use tracing::Level;

wasm_bindgen_test_configure!(run_in_browser);

/// Android WebView user agent — the platform gate matches "android" from
/// the UA before the Tauri check.
const ANDROID_UA: &str = "Mozilla/5.0 (Linux; Android 15; Pixel 6) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36";

fn events_about(marker: &str) -> Vec<(Level, String)> {
    log_capture::take_events()
        .into_iter()
        .filter(|(_, message)| message.contains(marker))
        .collect()
}

#[wasm_bindgen_test]
async fn caps_failure_on_android_escalates_to_error_event() {
    log_capture::install();
    reset_cached_capabilities();
    // In-flight caps queries from earlier suites (device_ai invoke tests)
    // reject on the JS event loop — their warnings land in whatever test
    // window is open at that moment. Let the stragglers settle, then start
    // the verdict window drained (the log_capture protocol: drain right
    // before driving the behaviour under test).
    gloo_timers::future::TimeoutFuture::new(50).await;
    log_capture::take_events();
    let _ua = UserAgentOverride::install(ANDROID_UA);
    // Reject the caps query — the plugin "present but failing" scenario.
    let _mock = TauriMock::install(|command, _args| {
        let command = command.as_string().unwrap_or_default();
        if command.ends_with("get_capabilities") {
            rejected_with("[PLUGIN_FAILURE]", "caps query failed")
        } else {
            rejected_with("[UNEXPECTED]", &command)
        }
    });

    // False is the expected routing outcome (fallback), the assertion below
    // is about the observability level accompanying it.
    let routed = available(Feature::TextRecognition).await;
    assert!(!routed, "failed caps query must route to fallback");

    // Count by THIS test's rejection marker, not by the generic phrase:
    // the shared log buffer accumulates caps-failure events from the other
    // device_ai tests (availability checks, real-invoke probes) that do not
    // drain it, so the generic phrase under-counts by pollution.
    let all = log_capture::take_events();
    let caps_events: Vec<_> = all
        .iter()
        .filter(|(_, message)| message.contains("[PLUGIN_FAILURE]"))
        .cloned()
        .collect();
    assert_eq!(
        caps_events.len(),
        1,
        "exactly one caps-failure event expected, all events: {all:?}"
    );
    assert_eq!(
        caps_events[0].0,
        Level::ERROR,
        "on android a caps failure must be an error (Sentry event), got: {:?}",
        caps_events[0]
    );
}

#[wasm_bindgen_test]
async fn caps_failure_on_web_stays_warn_without_error_event() {
    log_capture::install();
    reset_cached_capabilities();
    // Same settle+drain as the android test above: this test's window must
    // not absorb straggler warnings from earlier suites' in-flight caps
    // queries (observed on CI: 4 leftover "capabilities query failed"
    // events from the invoke suite landed here and broke the == 1 count).
    gloo_timers::future::TimeoutFuture::new(50).await;
    log_capture::take_events();
    // No UA override: the runner's browser UA is a desktop one and without
    // a Tauri shell platform_name() resolves to "web" — the plugin-less
    // platform where fallback routing is normal, not a degradation.
    let _mock = TauriMock::install(|command, _args| {
        let command = command.as_string().unwrap_or_default();
        if command.ends_with("get_capabilities") {
            rejected_with("[PLUGIN_FAILURE]", "caps query failed")
        } else {
            rejected_with("[UNEXPECTED]", &command)
        }
    });

    let routed = available(Feature::TextRecognition).await;
    assert!(!routed, "failed caps query must route to fallback");

    // Count by THIS test's rejection marker, not by the generic phrase:
    // the shared log buffer accumulates caps-failure events from the other
    // device_ai tests (availability checks, real-invoke probes) that do not
    // drain it, so the generic phrase under-counts by pollution.
    let all = log_capture::take_events();
    let caps_events: Vec<_> = all
        .iter()
        .filter(|(_, message)| message.contains("[PLUGIN_FAILURE]"))
        .cloned()
        .collect();
    assert_eq!(
        caps_events.len(),
        1,
        "exactly one caps-failure event expected, all events: {all:?}"
    );
    assert_eq!(
        caps_events[0].0,
        Level::WARN,
        "on web a caps failure must stay a warn (breadcrumb), got: {:?}",
        caps_events[0]
    );
}

#[wasm_bindgen_test]
async fn cached_caps_skip_the_query_and_its_event() {
    log_capture::install();
    reset_cached_capabilities();
    let _ua = UserAgentOverride::install(ANDROID_UA);
    let _mock = TauriMock::install(|command, _args| {
        let command = command.as_string().unwrap_or_default();
        if command.ends_with("get_capabilities") {
            all_available_caps()
        } else {
            rejected_with("[UNEXPECTED]", &command)
        }
    });

    assert!(
        available(Feature::TextRecognition).await,
        "available caps must route native"
    );
    log_capture::take_events();

    // Second call hits the cache: no repeated caps event noise.
    assert!(available(Feature::TextRecognition).await);
    let caps_events = events_about("capabilities query");
    assert!(
        caps_events.is_empty(),
        "cached capabilities must not re-emit caps events, got: {caps_events:?}"
    );
}
