//! Integration tests for the `security_headers` middleware.
//!
//! The middleware is registered as the outermost layer so the four defensive
//! headers (`X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`,
//! `Permissions-Policy`) reach every response class — most notably the 308
//! short-circuit from `strip_trailing_slash` and the 404 from the fallback
//! chain, which never pass through the inner stack.

#![cfg(feature = "ssr")]

use axum::body::Body;
use http::{Request, StatusCode, header::HeaderName};
use http_body_util::BodyExt;
use tower::ServiceExt;

mod common;

/// Drain the response and return `(status, header_values)` for the four
/// security headers. Missing headers surface as `None` so the per-test
/// assertions fail with a precise message rather than a panic inside the
/// helper.
///
/// `HeaderName::from_static` is used instead of the `http::header::X_*`
/// constants so the test does not depend on which header names the pinned
/// `http` crate version exposes (e.g. `PERMISSIONS_POLICY` was added later).
async fn security_headers_for(uri: &str) -> (StatusCode, [Option<String>; 4]) {
    let response = common::test_router()
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("router responded");

    let status = response.status();
    let names = [
        HeaderName::from_static("x-content-type-options"),
        HeaderName::from_static("x-frame-options"),
        HeaderName::from_static("referrer-policy"),
        HeaderName::from_static("permissions-policy"),
    ];
    let values = names.map(|name| {
        response
            .headers()
            .get(&name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    });

    let _ = response.into_body().collect().await.expect("body");
    (status, values)
}

fn assert_all_present(values: &[Option<String>; 4], context: &str) {
    let expected = [
        ("X-Content-Type-Options", "nosniff"),
        ("X-Frame-Options", "SAMEORIGIN"),
        ("Referrer-Policy", "strict-origin-when-cross-origin"),
        ("Permissions-Policy", "camera=(), microphone=(), geolocation=()"),
    ];
    for (got, (name, want)) in values.iter().zip(expected) {
        assert_eq!(
            got.as_deref(),
            Some(want),
            "{name} mismatch on {context}; got {got:?}"
        );
    }
}

#[tokio::test]
async fn html_root_has_security_headers() {
    let (status, values) = security_headers_for("/").await;
    assert_eq!(status, StatusCode::OK);
    assert_all_present(&values, "GET /");
}

#[tokio::test]
async fn trailing_slash_redirect_has_security_headers() {
    // The decisive outermost-layer check: `strip_trailing_slash` emits the 308
    // without calling `next`, so the headers can only reach the response if
    // `security_headers` sits outside it.
    let (status, values) = security_headers_for("/ru/").await;
    assert_eq!(status, StatusCode::PERMANENT_REDIRECT);
    assert_all_present(&values, "GET /ru/ (308)");
}

#[tokio::test]
async fn not_found_has_security_headers() {
    let (status, values) = security_headers_for("/random").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_all_present(&values, "GET /random (404)");
}

#[tokio::test]
async fn static_asset_has_security_headers() {
    let (status, values) = security_headers_for("/favicon.png").await;
    assert_eq!(status, StatusCode::OK);
    assert_all_present(&values, "GET /favicon.png");
}
