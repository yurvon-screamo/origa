//! Integration tests for HTTP 404 (soft-404 fix).
//!
//! Behaviour under test: unknown URLs return HTTP 404 (Not Found) so that
//! search engines treat them as deleted rather than indexing them as real
//! pages. The body must contain the visible "404" marker.

#![cfg(feature = "ssr")]

use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

mod common;

async fn get_status_and_body(uri: &str) -> (StatusCode, String) {
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
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let text = String::from_utf8_lossy(&body).into_owned();
    (status, text)
}

#[tokio::test]
async fn unknown_root_path_returns_404() {
    // Act
    let (status, body) = get_status_and_body("/random").await;

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND, "body was: {body}");
    assert!(body.contains("404"), "body should mention 404, got: {body}");
}

#[tokio::test]
async fn unknown_locale_subpath_returns_404() {
    // Act
    let (status, body) = get_status_and_body("/ru/random").await;

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("404"), "body should mention 404, got: {body}");
}

#[tokio::test]
async fn deep_unknown_path_returns_404() {
    // Act
    let (status, body) = get_status_and_body("/deep/nonexistent/path").await;

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("404"), "body should mention 404, got: {body}");
}

#[tokio::test]
async fn homepage_still_returns_200() {
    // Regression: homepage must keep working.
    let (status, _body) = get_status_and_body("/").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn locale_home_still_returns_200() {
    // Regression: locale homepage must keep working.
    let (status, _body) = get_status_and_body("/ru").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn favicon_still_returns_200() {
    // Regression: static asset must keep working.
    let (status, _body) = get_status_and_body("/favicon.png").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn sitemap_still_returns_200() {
    // Regression: sitemap must keep working.
    let (status, _body) = get_status_and_body("/sitemap.xml").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn unsupported_favicon_format_returns_404() {
    // `/favicon.webp` is not a published asset. The fallback `ServeDir` and
    // `ErrorHandler` must produce a real 404 (not a soft-200 shell), and the
    // `enforce_cache_policy` middleware must override any inner
    // `Cache-Control` to `no-cache` so a later-added format is served
    // immediately.
    let (status, body) = get_status_and_body("/favicon.webp").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("404"), "body should mention 404, got: {body}");
}

#[rstest::rstest]
#[case::en("/", "en")]
#[case::ru("/ru", "ru")]
#[case::ko("/ko", "ko")]
#[case::vi("/vi", "vi")]
#[tokio::test]
async fn html_lang_attribute_reflects_locale_in_ssr(
    #[case] path: &str,
    #[case] expected_lang: &str,
) {
    // Act
    let (_status, body) = get_status_and_body(path).await;

    // Assert: the SSR-rendered <html> tag must carry the locale's lang code so
    // that search engines and screen readers treat the page as the right
    // language. `leptos_meta::Html` inside `Layout` is the single source of
    // truth (see ADR-011 "<html lang> policy").
    assert!(
        body.contains(&format!("lang=\"{expected_lang}\"")),
        "expected lang=\"{expected_lang}\" on <html>, got first 500 chars: {}",
        body.chars().take(500).collect::<String>()
    );

    // Negative assertion: there must be exactly ONE lang attribute on <html>.
    // A duplicate (e.g. `<html lang="ru" lang="en">`) is invalid HTML5 and
    // would indicate that a hardcoded lang in the shell wasn't removed.
    let html_open_start = body
        .find("<html")
        .expect("response must contain an <html opening tag");
    let html_open_end = body[html_open_start..]
        .find('>')
        .map(|offset| html_open_start + offset)
        .expect("<html opening tag must close");
    let html_open_tag = &body[html_open_start..=html_open_end];
    let lang_count = html_open_tag.matches("lang=").count();
    assert_eq!(
        lang_count, 1,
        "expected exactly one lang= attribute on <html>, got {lang_count} in: {html_open_tag}"
    );
}
