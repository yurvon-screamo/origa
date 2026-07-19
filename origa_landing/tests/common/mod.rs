//! Shared helpers for `origa_landing` integration tests.
//!
//! Each file in `tests/` compiles as a separate integration-test binary, so
//! code reused across them must live in a module rather than a `#[path]`
//! import. `tests/common/mod.rs` is the conventional location and is NOT
//! picked up by Cargo as its own test target (only direct children of
//! `tests/` are). Each test file pulls it in with `mod common;`.

use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use leptos::config::LeptosOptions;
use tower::ServiceExt;

/// Build the exact same Axum router the production binary serves, so tests
/// exercise the real middleware stack (`strip_trailing_slash`,
/// `enforce_cache_policy`) and the real route table.
pub fn test_router() -> axum::Router {
    let opts = LeptosOptions::builder()
        .output_name("origa_landing")
        .build();
    origa_landing::server::build_router(opts)
}

/// Issue a GET against the test router and return `(status, body)`. The
/// canonical way to make a request in an integration test; reuses the
/// real middleware stack so redirects, cache headers, and the 404 fallback
/// chain all run as in production.
///
/// `#[allow(dead_code)]` because each `tests/*.rs` file compiles as its own
/// binary — only some test files exercise `get`, and an unused-fn warning
/// in the others would be noise.
#[allow(dead_code)]
pub async fn get(uri: &str) -> (StatusCode, String) {
    let response = test_router()
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("router responded");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    (status, text)
}
