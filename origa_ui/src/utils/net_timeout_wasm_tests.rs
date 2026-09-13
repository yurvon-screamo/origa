//! WASM tests for the idle-deadline network primitive (ADR-052).
//!
//! The stream-reader core is exercised directly with hand-built
//! ReadableStreams: a stalled stream (never enqueues) must abort with a
//! recognizable idle-timeout error, and a slow-but-alive stream (chunks
//! spaced wider than the deadline) must complete — proving chunk progress
//! resets the watchdog. Real `fetch` against a closed port verifies the
//! non-timeout failure path stays distinguishable.

#![cfg(all(target_arch = "wasm32", test))]

use wasm_bindgen_test::*;

use super::read_all_with_idle;
use crate::utils::now_ms;
use leptos::wasm_bindgen::{JsCast, JsValue};
use origa::domain::OrigaError;

wasm_bindgen_test_configure!(run_in_browser);

/// A ReadableStream that enqueues `chunks` single-byte chunks every
/// `period_ms` and then closes — a slow-but-alive transfer body.
fn stream_with_periodic_chunks(period_ms: u32, chunks: u32) -> web_sys::ReadableStream {
    let js = format!(
        r#"
        new ReadableStream({{
            start(controller) {{
                let i = 0;
                const timer = setInterval(() => {{
                    if (i < {chunks}) {{
                        controller.enqueue(new Uint8Array([i]));
                        i += 1;
                    }} else {{
                        clearInterval(timer);
                        controller.close();
                    }}
                }}, {period_ms});
            }}
        }})
        "#,
    );
    let value = js_sys::eval(&js).expect("stream constructor eval");
    value
        .dyn_into::<web_sys::ReadableStream>()
        .expect("eval result is a ReadableStream")
}

#[wasm_bindgen_test]
async fn stalled_stream_aborts_with_the_idle_timeout_marker() {
    // A stream with no source never enqueues and never closes: the
    // watchdog must abort it (not await forever) and mark the error.
    let started = now_ms();
    let stream = web_sys::ReadableStream::new().expect("empty stream");

    let error = read_all_with_idle(stream, "probe://stalled", 100)
        .await
        .expect_err("a stalled stream must not complete");

    assert!(
        super::is_idle_timeout(&error),
        "expected an idle-timeout error, got: {error:?}"
    );
    let elapsed = now_ms() - started;
    assert!(
        (90.0..=3000.0).contains(&elapsed),
        "abort must happen shortly after the 100 ms deadline, took {elapsed} ms"
    );
}

#[wasm_bindgen_test]
async fn chunk_progress_resets_the_idle_deadline() {
    // Chunks arrive every 60 ms under a 100 ms deadline, and the whole
    // transfer takes ~300 ms: an aggregate deadline would abort at
    // 100 ms, but the idle watchdog must restart on every chunk and let
    // the progressing stream finish.
    let stream = stream_with_periodic_chunks(60, 5);

    let bytes = read_all_with_idle(stream, "probe://slow", 100)
        .await
        .expect("a progressing stream must outlive the idle budget chunk by chunk");

    assert_eq!(bytes, vec![0, 1, 2, 3, 4]);
}

#[wasm_bindgen_test]
fn idle_timeout_errors_are_recognizable_among_network_errors() {
    let timeout = OrigaError::NetworkError {
        url: "probe://stalled".to_string(),
        reason: "idle timeout after 30 ms without data".to_string(),
    };
    assert!(super::is_idle_timeout(&timeout));

    let http = OrigaError::NetworkError {
        url: "probe://server".to_string(),
        reason: "HTTP 500".to_string(),
    };
    assert!(!super::is_idle_timeout(&http));
}

#[wasm_bindgen_test]
async fn refused_connection_fails_fast_without_the_timeout_marker() {
    // Port 9 (discard) has no listener: the browser rejects the fetch
    // immediately. The error must NOT look like an idle timeout —
    // that distinction feeds the retry gate (ADR-052).
    let error = super::fetch_idle("http://127.0.0.1:9/probe", 2_000)
        .await
        .expect_err("closed port must fail");

    assert!(!super::is_idle_timeout(&error));
}

#[wasm_bindgen_test]
async fn refused_api_post_fails_as_a_network_error() {
    // The TrailBase transport sends method/headers/body via
    // send_request_idle; a dead API endpoint must surface as a network
    // error (fast refusal, not the idle-timeout marker) so login and
    // session flows fail fast offline instead of hanging.
    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_body(&JsValue::from_str(r#"{"email":"x@y.z"}"#));

    let error = super::send_request_idle("http://127.0.0.1:9/api/auth/v1/login", &init, 2_000)
        .await
        .expect_err("closed API port must fail");

    assert!(!super::is_idle_timeout(&error));
}
