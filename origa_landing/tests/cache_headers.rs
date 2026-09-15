//! Integration tests for `Cache-Control` headers.
//!
//! Behaviour under test:
//! - HTML pages (leptos routes): `public, max-age=300` (5-minute edge cache).
//! - Static hashed assets (favicon, /images/*, CSS): `public, max-age=31536000, immutable`.
//! - Crawl-control files (robots.txt, sitemap.xml): `no-cache`.
//! - Error responses (4xx/5xx): `no-cache`, never `immutable`. Without the
//!   `enforce_cache_policy` middleware, `ServeDir`/`ServeFile` would stamp
//!   `IMMUTABLE_CACHE` on a 404, and the CDN would pin "not found" for a year.

#![cfg(feature = "ssr")]

use axum::body::Body;
use http::{
    Request, StatusCode,
    header::{CACHE_CONTROL, VARY},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

mod common;

async fn cache_control(uri: &str) -> Option<String> {
    // Delegate to `status_and_cache_control` so the router→oneshot→drain
    // sequence lives in exactly one place (DRY). Callers that don't need
    // the status discard it via this thin wrapper.
    status_and_cache_control(uri).await.1
}

async fn status_and_cache_control(uri: &str) -> (StatusCode, Option<String>) {
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
    let cc = response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    let _ = response.into_body().collect().await.expect("body");
    (status, cc)
}

#[tokio::test]
async fn html_root_has_short_edge_cache_and_locale_vary() {
    // "/" is content-negotiated by `negotiate_locale`, so its 200 must carry
    // both the short edge cache and a `Vary` on the negotiation headers.
    // Without `Vary`, an edge-cached English "/" would be served to
    // non-English visitors before the redirect can fire (ADR-011 / PR #182).
    let response = common::test_router()
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("router responded");

    let cc = response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let vary = response
        .headers()
        .get(VARY)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    let _ = response.into_body().collect().await.expect("body");
    assert_eq!(cc.as_deref(), Some("public, max-age=300"));
    assert_eq!(vary.as_deref(), Some("Cookie, Accept-Language"));
}

#[tokio::test]
async fn html_locale_path_has_short_edge_cache() {
    let cc = cache_control("/ru").await;
    assert_eq!(cc.as_deref(), Some("public, max-age=300"));
}

#[tokio::test]
async fn favicon_has_immutable_cache() {
    let cc = cache_control("/favicon.png").await;
    assert_eq!(cc.as_deref(), Some("public, max-age=31536000, immutable"));
}

#[tokio::test]
async fn favicon_ico_has_immutable_cache() {
    let cc = cache_control("/favicon.ico").await;
    assert_eq!(cc.as_deref(), Some("public, max-age=31536000, immutable"));
}

#[tokio::test]
async fn apple_touch_icon_has_immutable_cache() {
    let cc = cache_control("/apple-touch-icon.png").await;
    assert_eq!(cc.as_deref(), Some("public, max-age=31536000, immutable"));
}

#[tokio::test]
async fn browserconfig_xml_has_immutable_cache() {
    // Windows Edge/IE tile config reuses the 180x180 apple-touch-icon; the
    // file only changes when the logo does, so immutable caching matches the
    // favicon policy. See ADR-016.
    let cc = cache_control("/browserconfig.xml").await;
    assert_eq!(cc.as_deref(), Some("public, max-age=31536000, immutable"));
}

#[tokio::test]
async fn static_css_has_immutable_cache() {
    let cc = cache_control("/landing.processed.css").await;
    assert_eq!(cc.as_deref(), Some("public, max-age=31536000, immutable"));
}

#[tokio::test]
async fn image_has_immutable_cache() {
    // logo.png ships in the repo (see origa_landing/public/images/).
    let cc = cache_control("/images/logo.png").await;
    assert_eq!(cc.as_deref(), Some("public, max-age=31536000, immutable"));
}

/// Same-origin landing web fonts (@font-face rules in style/input.css,
/// files committed under public/fonts/landing/ and served immutable via
/// the /fonts nest_service). The test below DERIVES the expected font list
/// from input.css, so an @font-face added without its woff2 file fails
/// here instead of 404-ing in production — the exact failure mode of the
/// 2026-09 incident this test guards against.
const INPUT_CSS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/style/input.css");

/// Extract every `/fonts/landing/…` URL referenced by a `src: url("…")` in
/// input.css. Manual scan (no regex): each match starts at `url("/fonts/`
/// and ends at the closing `")`.
fn landing_font_urls() -> Vec<String> {
    let css = std::fs::read_to_string(INPUT_CSS).expect("style/input.css is readable");
    let mut urls = Vec::new();
    let mut rest = css.as_str();
    while let Some(start) = rest.find("url(\"/fonts/") {
        let after = &rest[start + "url(\"".len()..];
        let Some(end) = after.find("\")") else {
            break;
        };
        urls.push(after[..end].to_string());
        rest = &after[end..];
    }
    urls
}

#[tokio::test]
async fn every_input_css_font_face_resolves_to_committed_file() {
    let urls = landing_font_urls();
    assert!(
        urls.len() >= 10,
        "input.css references {} landing fonts; expected at least the 10 \
         shipped ones — did the @font-face block move or get renamed?",
        urls.len()
    );

    let mut broken = Vec::new();
    for path in &urls {
        let (status, cc) = status_and_cache_control(path).await;
        if status != StatusCode::OK {
            broken.push(format!(
                "{path}: status {status} (file missing from public/?):"
            ));
        } else if cc.as_deref() != Some("public, max-age=31536000, immutable") {
            broken.push(format!("{path}: not immutable ({cc:?})"));
        }
    }
    assert!(
        broken.is_empty(),
        "input.css @font-face references that do not resolve to a committed, \
         immutably-served file:\n{}",
        broken.join("\n")
    );
}

#[tokio::test]
async fn stylesheet_href_keeps_cache_bust_suffix() {
    // The stylesheet is served immutable; app.rs MUST reference it with a
    // `?v=` cache-bust suffix or returning visitors keep stale CSS for a
    // year. This only guards the suffix itself — bumping `v` on every CSS
    // change remains a manual contract (see the comment at the href).
    let app_rs = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .expect("src/app.rs is readable");
    assert!(
        app_rs.contains("landing.processed.css?v="),
        "stylesheet href lost its ?v= cache-bust suffix"
    );
}

#[tokio::test]
async fn versioned_css_query_serves_immutable() {
    // app.rs pins the stylesheet with a `?v=` cache-bust suffix (immutable
    // contract: bump v on ANY css change). The route matches on path only,
    // so the query-carrying URL must be served by the same ServeFile.
    let (status, cc) = status_and_cache_control("/landing.processed.css?v=20260915").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cc.as_deref(), Some("public, max-age=31536000, immutable"));
}

#[tokio::test]
async fn robots_txt_has_no_cache() {
    let cc = cache_control("/robots.txt").await;
    assert_eq!(cc.as_deref(), Some("no-cache"));
}

#[tokio::test]
async fn sitemap_xml_has_no_cache() {
    let cc = cache_control("/sitemap.xml").await;
    assert_eq!(cc.as_deref(), Some("no-cache"));
}

#[tokio::test]
async fn llms_txt_has_no_cache() {
    // llms.txt summarises the product for AI assistants/crawlers. Unlike a
    // hashed asset, its copy is updated per release, so immutable caching would
    // pin stale text at the CDN edge (the same edge-poisoning class of bug as
    // PR #182). no-cache keeps it always-fresh. See ADR-016.
    let cc = cache_control("/llms.txt").await;
    assert_eq!(cc.as_deref(), Some("no-cache"));
}

#[tokio::test]
async fn indexnow_key_file_has_no_cache() {
    // The IndexNow key file proves domain ownership. Search engines fetch it
    // on first submission; if it were immutable-cached and the key rotated,
    // verification would fail until the edge cache expired. no-cache ensures
    // the current key is always served. See ADR-038.
    let cc = cache_control("/e7825074-6888-4e03-a9ad-91459e4c9940.txt").await;
    assert_eq!(cc.as_deref(), Some("no-cache"));
}

#[tokio::test]
async fn missing_image_404_is_not_cached_as_immutable() {
    // Regression for the SEO "Common-1" issue: `ServeDir` stamps
    // `IMMUTABLE_CACHE` on its 404 via `insert_response_header_if_not_present`
    // (which fires on *all* statuses). The `enforce_cache_policy` middleware
    // must override that to `NO_CACHE` so the CDN does not pin "not found"
    // for a year — otherwise a later-added image would not be served until
    // the cache expired.
    let (status, cc) = status_and_cache_control("/images/definitely-missing.png").await;

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        cc.as_deref(),
        Some("no-cache"),
        "404 must carry no-cache, not immutable; got {cc:?}"
    );
}

#[tokio::test]
async fn missing_fallback_file_404_is_not_cached_as_immutable() {
    // Defence-in-depth: any path that resolves to a missing file under the
    // fallback `ServeDir(public/)` must also return 404 without immutable
    // caching. This response comes from `ErrorHandler` (leptos NotFound),
    // which sets no `Cache-Control` itself; `enforce_cache_policy` must
    // still force `NO_CACHE` on the 4xx. The assertion is exact (not
    // `assert_ne!(immutable)`) so a regression where the middleware stops
    // stamping `no-cache` on this code path is caught — `None` would fail
    // the equality, not just differ from immutable.
    let (status, cc) = status_and_cache_control("/no-such-file.txt").await;

    // Assert
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        cc.as_deref(),
        Some("no-cache"),
        "404 must carry no-cache; got {cc:?}"
    );
}
