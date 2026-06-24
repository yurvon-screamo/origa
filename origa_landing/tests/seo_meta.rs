//! Integration tests for SSR-rendered SEO metadata: Schema.org JSON-LD,
//! Open Graph tags, default <title>, and ARIA on decorative elements.
//!
//! Tests render the production router via `tower::ServiceExt::oneshot`
//! (same pattern as `not_found.rs`/`cache_headers.rs`) and parse the
//! resulting HTML with plain string assertions — the existing convention
//! for this crate. No HTML-parsing crate is pulled in to keep dev-deps
//! minimal.

#![cfg(feature = "ssr")]

use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

mod common;

async fn get_body(uri: &str) -> String {
    // Delegate status handling so the 200-assertion lives in one place; the
    // 404 path used by the default-title test bypasses it via `get_body_any`.
    let (status, body) = get_body_any(uri).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "expected 200 for {uri}, body was: {body}"
    );
    body
}

async fn get_body_any(uri: &str) -> (StatusCode, String) {
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

/// Extract the first `application/ld+json` script body from `html`. Returns
/// the raw JSON text (without the wrapping `<script>` tags).
fn first_jsonld_block(html: &str) -> String {
    let open = r#"<script type="application/ld+json">"#;
    let close = "</script>";
    let start = html
        .find(open)
        .unwrap_or_else(|| panic!("no JSON-LD block in body: {}", &html[..html.len().min(400)]))
        + open.len();
    let end = html[start..]
        .find(close)
        .unwrap_or_else(|| panic!("JSON-LD block not closed: {}", &html[start..start + 400]))
        + start;
    html[start..end].to_owned()
}

#[tokio::test]
async fn ru_home_schema_description_is_russian_not_english() {
    let body = get_body("/ru").await;
    let json = first_jsonld_block(&body);
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON-LD must be valid JSON");

    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .expect("SoftwareApplication schema must have a description");
    assert!(
        description.contains("лексика") || description.contains("кандзи"),
        "RU description should be Russian; got: {description}"
    );
}

#[tokio::test]
async fn software_application_schema_in_language_is_single_string() {
    // sitemaps.org / schema.org `inLanguage` should be a single BCP-47
    // string, not an array — Google's rich-results validator flags arrays.
    let body = get_body("/").await;
    let json = first_jsonld_block(&body);
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON-LD must parse");

    let in_language = value
        .get("inLanguage")
        .expect("schema must have inLanguage");
    assert!(
        in_language.is_string(),
        "inLanguage must be a string, got: {in_language}"
    );
    assert_eq!(in_language.as_str(), Some("en"));
}

#[tokio::test]
async fn software_application_schema_has_feature_list() {
    let body = get_body("/ko").await;
    let json = first_jsonld_block(&body);
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON-LD must parse");

    let feature_list = value
        .get("featureList")
        .and_then(|v| v.as_str())
        .expect("schema must have featureList");
    assert!(
        feature_list.contains("한자"),
        "KO featureList must be Korean; got: {feature_list}"
    );
}

#[tokio::test]
async fn how_to_schema_has_in_language() {
    let body = get_body("/features").await;
    let json = first_jsonld_block(&body);
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON-LD must parse");

    assert_eq!(value.get("@type").and_then(|v| v.as_str()), Some("HowTo"));
    assert_eq!(
        value.get("inLanguage").and_then(|v| v.as_str()),
        Some("en"),
        "HowTo schema must carry inLanguage; got: {value}"
    );
}

#[tokio::test]
async fn vi_how_to_schema_name_is_vietnamese() {
    let body = get_body("/vi/features").await;
    let json = first_jsonld_block(&body);
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON-LD must parse");

    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .expect("HowTo schema must have a name");
    assert!(
        name.contains("tiếng Nhật"),
        "VI HowTo name must be Vietnamese; got: {name}"
    );
}

#[tokio::test]
async fn jsonld_block_parses_for_all_locales() {
    // Regression for the `</script>` XSS defence: the `<` escaping must not
    // break JSON validity. If the escape is malformed, `serde_json` rejects
    // the payload with a syntax error.
    for uri in ["/", "/ru", "/ko", "/vi", "/features", "/ru/features"] {
        let body = get_body(uri).await;
        let json = first_jsonld_block(&body);
        serde_json::from_str::<serde_json::Value>(&json)
            .unwrap_or_else(|e| panic!("JSON-LD on {uri} must parse: {e}\nblock: {json}"));
    }
}

#[tokio::test]
async fn jsonld_escapes_less_than_sign() {
    // The XSS defence replaces `<` with `\u003c`. None of our current copy
    // contains `<`, but the property must hold generically: render a page
    // and assert no raw `<` survives inside the JSON-LD body (only valid
    // JSON escapes are allowed).
    let body = get_body("/").await;
    let json = first_jsonld_block(&body);
    assert!(
        !json.contains('<'),
        "raw `<` inside JSON-LD must be escaped; got: {json}"
    );
}

#[tokio::test]
async fn og_locale_alternates_present_for_other_locales() {
    let body = get_body("/ru").await;
    // og:locale (current) must NOT include alternates for itself, only others.
    assert!(
        body.contains(r#"property="og:locale" content="ru_RU""#),
        "current og:locale missing; got first 600 chars: {}",
        &body[..body.len().min(600)]
    );
    for alt in ["en_US", "ko_KR", "vi_VN"] {
        assert!(
            body.contains(&format!(
                r#"property="og:locale:alternate" content="{alt}""#
            )),
            "og:locale:alternate {alt} missing"
        );
    }
    // The current locale must NOT appear as an alternate of itself.
    let ru_alt_count = body
        .matches(r#"property="og:locale:alternate" content="ru_RU""#)
        .count();
    assert_eq!(
        ru_alt_count, 0,
        "current locale must not be listed as its own alternate"
    );
}

#[tokio::test]
async fn en_home_has_three_alternates() {
    let body = get_body("/").await;
    let alt_count = body.matches(r#"property="og:locale:alternate""#).count();
    // EN + RU + KO + VI = 4 locales; minus the current = 3 alternates.
    assert_eq!(
        alt_count, 3,
        "expected 3 og:locale:alternate tags, got {alt_count}"
    );
}

#[tokio::test]
async fn default_title_is_descriptive() {
    // The shell sets a default <title> in App(); page-level PageMeta overrides
    // it on real routes. The only route without PageMeta is the NotFound
    // branch, so the default title is what shows up on 404. A bare "Origa"
    // here would be a title-quality regression for any soft-404 a crawler
    // happens to discover.
    let (status, body) = get_body_any("/random-nonexistent").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        body.contains("<title>Origa — Japanese Learning App</title>"),
        "default title not found; got first 800 chars: {}",
        &body[..body.len().min(800)]
    );
}

#[tokio::test]
async fn features_hero_decor_has_aria_hidden() {
    // Decorative background-image elements must not be exposed to assistive
    // tech — the visible <h1> already conveys the section's purpose.
    //
    // The assertion is attribute-order independent: Leptos SSR does not
    // guarantee that `aria-hidden` follows `class`/`style` in the rendered
    // tag, so we locate the entire enclosing `<div ...>` opening tag (from the
    // nearest preceding `<` to the next `>`) and check it carries the
    // attribute anywhere within.
    let body = get_body("/features").await;
    let class_idx = body
        .find("feat-hero__decor-img")
        .expect("feat-hero__decor-img must be rendered");
    let tag_start = body[..class_idx]
        .rfind('<')
        .expect("opening '<' must precede feat-hero__decor-img");
    let tag_end = body[class_idx..]
        .find('>')
        .map(|offset| class_idx + offset)
        .expect("decor div opening tag must close");
    let decor_open_tag = &body[tag_start..=tag_end];
    assert!(
        decor_open_tag.contains(r#"aria-hidden="true""#),
        "decorative background-image div must carry aria-hidden; got: {decor_open_tag}"
    );
}
