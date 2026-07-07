//! Integration tests for trailing-slash canonicalisation.
//!
//! Behaviour under test: the `strip_trailing_slash` middleware redirects
//! `/path/` to `/path` with HTTP 308 Permanent Redirect. The site root `/`
//! is exempt by convention. Permanent redirects carry
//! `Cache-Control: public, max-age=86400` so CDNs absorb the redirect
//! without re-hitting origin on every crawl.

#![cfg(feature = "ssr")]

use axum::body::Body;
use http::{Method, Request, StatusCode, header::CACHE_CONTROL, header::LOCATION};
use http_body_util::BodyExt;
use tower::ServiceExt;

mod common;

async fn send(uri: &str, method: Method) -> (StatusCode, Option<String>, String) {
    let response = common::test_router()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("router responded");

    let status = response.status();
    let location = response
        .headers()
        .get(LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body_text = String::from_utf8_lossy(&body).into_owned();
    (status, location, body_text)
}

/// Same as `send` but also captures `Cache-Control` (and drops the body, which
/// redirect tests never inspect). Used by tests that verify the redirect's
/// cache policy.
async fn send_with_cache(
    uri: &str,
    method: Method,
) -> (StatusCode, Option<String>, Option<String>) {
    let response = common::test_router()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("router responded");

    let status = response.status();
    let location = response
        .headers()
        .get(LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let cache_control = response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    let _ = response.into_body().collect().await.expect("body");
    (status, location, cache_control)
}

#[rstest::rstest]
#[case::ru("/ru/", "/ru")]
#[case::ko("/ko/", "/ko")]
#[case::vi("/vi/", "/vi")]
#[tokio::test]
async fn locale_root_with_trailing_slash_redirects(
    #[case] input: &str,
    #[case] expected_location: &str,
) {
    // Act
    let (status, location, _body) = send(input, Method::GET).await;

    // Assert
    assert_eq!(status, StatusCode::PERMANENT_REDIRECT);
    assert_eq!(location.as_deref(), Some(expected_location));
}

#[rstest::rstest]
#[case("/ru/features/", "/ru/features")]
#[case("/ru/compare/", "/ru/compare")]
#[case("/ru/content/", "/ru/content")]
#[case("/ru/download/", "/ru/download")]
#[case("/ko/features/", "/ko/features")]
#[case("/vi/download/", "/vi/download")]
#[case("/privacy/", "/privacy")]
#[case("/terms/", "/terms")]
#[case("/ru/privacy/", "/ru/privacy")]
#[case("/ko/terms/", "/ko/terms")]
// English paths have no locale prefix — the canonical form is /features, not
// /en/features. These cases guard against a regression where only locale-
// prefixed paths were normalised.
#[case("/features/", "/features")]
#[case("/compare/", "/compare")]
#[case("/content/", "/content")]
#[case("/download/", "/download")]
#[tokio::test]
async fn locale_subpage_with_trailing_slash_redirects(
    #[case] input: &str,
    #[case] expected_location: &str,
) {
    // Act
    let (status, location, _body) = send(input, Method::GET).await;

    // Assert
    assert_eq!(status, StatusCode::PERMANENT_REDIRECT);
    assert_eq!(location.as_deref(), Some(expected_location));
}

#[tokio::test]
async fn root_with_trailing_slash_is_not_redirected() {
    // Act
    let (status, location, _body) = send("/", Method::GET).await;

    // Assert: `/` is the canonical site root; middleware must not touch it.
    assert_ne!(status, StatusCode::PERMANENT_REDIRECT);
    assert!(location.is_none());
}

#[rstest::rstest]
#[case("/ru")]
#[case("/ko")]
#[case("/vi")]
#[case("/ru/features")]
#[tokio::test]
async fn canonical_path_without_trailing_slash_is_not_redirected(#[case] input: &str) {
    // Act
    let (status, location, _body) = send(input, Method::GET).await;

    // Assert
    assert_ne!(status, StatusCode::PERMANENT_REDIRECT);
    assert!(location.is_none());
}

#[tokio::test]
async fn trailing_slash_redirect_preserves_query_string() {
    // Act
    let (status, location, _body) = send("/ru/?ref=twitter", Method::GET).await;

    // Assert
    assert_eq!(status, StatusCode::PERMANENT_REDIRECT);
    assert_eq!(location.as_deref(), Some("/ru?ref=twitter"));
}

#[tokio::test]
async fn head_request_with_trailing_slash_is_redirected() {
    // Act
    let (status, location, _body) = send("/ru/", Method::HEAD).await;

    // Assert: HEAD mirrors GET for redirects per RFC 7231.
    assert_eq!(status, StatusCode::PERMANENT_REDIRECT);
    assert_eq!(location.as_deref(), Some("/ru"));
}

#[tokio::test]
async fn post_request_with_trailing_slash_is_not_redirected() {
    // Act
    let (status, location, _body) = send("/ru/", Method::POST).await;

    // Assert: only safe methods get auto-redirected.
    assert_ne!(status, StatusCode::PERMANENT_REDIRECT);
    assert!(location.is_none());
}

#[tokio::test]
async fn multiple_trailing_slashes_collapse() {
    // Act
    let (status, location, _body) = send("/ru///", Method::GET).await;

    // Assert
    assert_eq!(status, StatusCode::PERMANENT_REDIRECT);
    assert_eq!(location.as_deref(), Some("/ru"));
}

#[tokio::test]
async fn static_asset_path_with_trailing_slash_redirects() {
    // Act
    let (status, location, _body) = send("/favicon.png/", Method::GET).await;

    // Assert: middleware fires before routing, so even unknown slash-suffixed
    // paths get normalised. What happens after the redirect (200 or 404) is
    // a separate concern.
    assert_eq!(status, StatusCode::PERMANENT_REDIRECT);
    assert_eq!(location.as_deref(), Some("/favicon.png"));
}

#[rstest::rstest]
#[case::locale_root("/ru/")]
#[case::another_locale("/ko/")]
#[case::with_query_string("/ru/?x=1")]
#[case::collapsed_slashes("/ru///")]
#[tokio::test]
async fn trailing_slash_redirect_is_cacheable(#[case] input: &str) {
    // Regression for the SEO "Common-2" issue: 308 redirects from
    // `strip_trailing_slash` are produced by the outermost layer and bypass
    // the inner cache-policy middleware, so they used to ship without any
    // `Cache-Control`. CDNs therefore re-hit origin on every crawl of a
    // slash-suffixed URL. The redirect must now carry a 24h edge cache so
    // Googlebot/Yandex/Bing pick up the canonical URL without an extra
    // origin round-trip on every visit.
    let (status, _location, cache_control) = send_with_cache(input, Method::GET).await;

    // Assert
    assert_eq!(status, StatusCode::PERMANENT_REDIRECT);
    assert_eq!(
        cache_control.as_deref(),
        Some("public, max-age=86400"),
        "permanent redirects must be cacheable for 24h; got {cache_control:?}"
    );
}

#[tokio::test]
async fn head_redirect_is_cacheable() {
    // HEAD mirrors GET for redirects per RFC 7231, including the cache policy.
    let (status, _location, cache_control) = send_with_cache("/ru/", Method::HEAD).await;

    assert_eq!(status, StatusCode::PERMANENT_REDIRECT);
    assert_eq!(
        cache_control.as_deref(),
        Some("public, max-age=86400"),
        "HEAD redirect must carry the same cache-control as GET"
    );
}
