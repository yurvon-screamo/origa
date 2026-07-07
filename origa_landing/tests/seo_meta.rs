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
    let start = html.find(open).unwrap_or_else(|| {
        panic!(
            "no JSON-LD block in body: {}",
            html.chars().take(400).collect::<String>()
        )
    }) + open.len();
    let end = html[start..].find(close).unwrap_or_else(|| {
        panic!(
            "JSON-LD block not closed: {}",
            html[start..].chars().take(400).collect::<String>()
        )
    }) + start;
    html[start..end].to_owned()
}

/// Find the first JSON-LD block whose `@type` matches `type_name`. Pages emit
/// several schemas (SoftwareApplication + Organization on the home page;
/// HowTo + BreadcrumbList + LearningResource on /features); this locates the
/// specific one under test so the assertion targets the right entity.
fn find_jsonld_block_by_type(html: &str, type_name: &str) -> String {
    let open = r#"<script type="application/ld+json">"#;
    let close = "</script>";
    let needle = format!("\"@type\":\"{type_name}\"");
    let mut rest = html;
    while let Some(start) = rest.find(open) {
        let body_start = start + open.len();
        let body_end = rest[body_start..]
            .find(close)
            .unwrap_or_else(|| panic!("JSON-LD block not closed"))
            + body_start;
        let block = &rest[body_start..body_end];
        if block.contains(&needle) {
            return block.to_owned();
        }
        rest = &rest[body_end + close.len()..];
    }
    panic!(
        "no JSON-LD block with @type={type_name}; body was: {}",
        html.chars().take(500).collect::<String>()
    )
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
        body.chars().take(600).collect::<String>()
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
        body.chars().take(800).collect::<String>()
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
    //
    // Two complementary checks guard against a regression where the RSX
    // `attr:aria-hidden` form leaks the `attr:` prefix into SSR HTML: the prefix
    // would satisfy a naive `contains("aria-hidden")` (false positive) while
    // browsers ignore the resulting invalid attribute.
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
        !decor_open_tag.contains("attr:aria-hidden"),
        "attr: prefix must not leak into SSR HTML; got: {decor_open_tag}"
    );
    assert!(
        decor_open_tag.contains(r#"aria-hidden="true""#),
        "decorative background-image div must carry aria-hidden; got: {decor_open_tag}"
    );
}

#[tokio::test]
async fn en_home_has_keywords_meta() {
    let body = get_body("/").await;
    assert!(
        body.contains(r#"<meta name="keywords" content="japanese"#),
        "EN keywords meta missing or not starting with a japanese keyword; got first 800 chars: {}",
        body.chars().take(800).collect::<String>()
    );
}

#[tokio::test]
async fn ru_keywords_are_russian() {
    let body = get_body("/ru").await;
    let kw = body
        .find(r#"<meta name="keywords" content=""#)
        .and_then(|i| {
            let start = i + r#"<meta name="keywords" content=""#.len();
            body[start..].find('"').map(|end| &body[start..start + end])
        })
        .expect("RU keywords meta must be present");
    assert!(
        kw.contains("кандзи"),
        "RU keywords must contain 'кандзи'; got: {kw}"
    );
    assert!(
        !kw.contains("japanese"),
        "RU keywords must not leak the English word 'japanese'; got: {kw}"
    );
}

#[tokio::test]
async fn vi_keywords_contain_han_tu() {
    let body = get_body("/vi").await;
    assert!(
        body.contains(
            r#"<meta name="keywords" content="học tiếng nhật, app học tiếng nhật, hán tự"#
        ),
        "VI keywords meta must contain 'hán tự'; got first 800 chars: {}",
        body.chars().take(800).collect::<String>()
    );
}

#[tokio::test]
async fn ko_keywords_are_korean() {
    let body = get_body("/ko").await;
    let has_korean =
        body.contains(r#"<meta name="keywords" content="일본어"#) || body.contains("한자");
    assert!(
        has_korean,
        "KO keywords meta must contain Korean text; got first 800 chars: {}",
        body.chars().take(800).collect::<String>()
    );
}

#[tokio::test]
async fn features_has_breadcrumb_schema() {
    let body = get_body("/features").await;
    let block = find_jsonld_block_by_type(&body, "BreadcrumbList");
    let value: serde_json::Value =
        serde_json::from_str(&block).expect("BreadcrumbList block must parse");
    let items = value
        .get("itemListElement")
        .and_then(|v| v.as_array())
        .expect("BreadcrumbList must have itemListElement array");
    assert_eq!(items.len(), 2, "breadcrumb must have home + current");
    let home_name = items[0]
        .get("name")
        .and_then(|v| v.as_str())
        .expect("home item must have a name");
    assert_eq!(home_name, "Home");
}

#[tokio::test]
async fn features_has_learning_resource_schema() {
    let body = get_body("/features").await;
    let block = find_jsonld_block_by_type(&body, "LearningResource");
    assert!(
        block.contains("\"isAccessibleForFree\":true"),
        "LearningResource must declare isAccessibleForFree:true; got: {block}"
    );
    let value: serde_json::Value =
        serde_json::from_str(&block).expect("LearningResource block must parse");
    let levels = value
        .get("educationalLevel")
        .and_then(|v| v.as_array())
        .expect("LearningResource must list educationalLevel");
    assert!(
        levels.iter().any(|l| l.as_str() == Some("JLPT N5")),
        "educationalLevel must cover JLPT N5–N1; got: {levels:?}"
    );
}

#[tokio::test]
async fn ru_features_learning_resource_teaches_is_russian() {
    // `teaches` is free-text per ADR-018, so it must follow the page locale:
    // the RU LearningResource teaches «Кандзи», not the English "Kanji".
    let body = get_body("/ru/features").await;
    let block = find_jsonld_block_by_type(&body, "LearningResource");
    let value: serde_json::Value =
        serde_json::from_str(&block).expect("LearningResource block must parse");
    let teaches = value
        .get("teaches")
        .and_then(|v| v.as_array())
        .expect("LearningResource must list teaches");
    let joined = teaches
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    assert!(
        joined.contains("Кандзи"),
        "RU teaches must be localised ('Кандзи'); got: {joined}"
    );
    assert!(
        !joined.contains("Kanji"),
        "RU teaches must not leak the English 'Kanji'; got: {joined}"
    );
}

#[tokio::test]
async fn home_org_has_sameas() {
    // The Organization block (second JSON-LD on the home page) must link to
    // the GitHub repo via sameAs so knowledge panels can connect the entity.
    let body = get_body("/").await;
    let block = find_jsonld_block_by_type(&body, "Organization");
    assert!(
        block.contains("\"sameAs\""),
        "Organization schema must carry sameAs; got: {block}"
    );
    assert!(
        block.contains("github.com/yurvon-screamo/origa"),
        "Organization sameAs must point at the GitHub repo; got: {block}"
    );
}

#[tokio::test]
async fn home_has_no_breadcrumb() {
    // The home page IS the breadcrumb root; emitting a BreadcrumbList there
    // (position 1 → position 1) is a schema error. Guard against an accidental
    // wiring of breadcrumb_schema on HomePage.
    let body = get_body("/").await;
    assert!(
        !body.contains("BreadcrumbList"),
        "home page must not emit a BreadcrumbList schema; got: {}",
        body.chars().take(500).collect::<String>()
    );
}

#[tokio::test]
async fn features_has_faq_schema() {
    let body = get_body("/features").await;
    let block = find_jsonld_block_by_type(&body, "FAQPage");
    let value: serde_json::Value = serde_json::from_str(&block).expect("FAQPage block must parse");
    let entities = value
        .get("mainEntity")
        .and_then(|v| v.as_array())
        .expect("FAQPage must have a mainEntity array");
    assert!(
        !entities.is_empty(),
        "FAQPage mainEntity must be non-empty; got: {entities:?}"
    );
    // Google requires every Question to carry an acceptedAnswer.
    for entity in entities {
        assert!(
            entity.get("acceptedAnswer").is_some(),
            "every FAQ Question must have an acceptedAnswer; got: {entity}"
        );
    }
}

#[tokio::test]
async fn features_has_visible_faq_block() {
    // Google's FAQPage policy requires the Q&A to be visible on the page, not
    // only in JSON-LD. Assert the first question renders inside the visible
    // <section class="feat-faq"> (i.e. after the section marker, not just
    // inside the <script> block).
    let body = get_body("/features").await;
    let section_idx = body
        .find(r#"<section class="feat-faq">"#)
        .expect("features page must render a visible .feat-faq section");
    let after_section = &body[section_idx..];
    assert!(
        after_section.contains("How do I start learning Japanese with Origa?"),
        "FAQ Q1 must appear in the visible .feat-faq section, not only in JSON-LD"
    );
}

#[tokio::test]
async fn footer_has_legal_links_on_every_locale() {
    // The Layout wraps every route, so the footer's Legal column must render
    // /privacy and /terms links on every locale variant of the home page.
    // Required for Google Play policy compliance (Privacy Policy reachable
    // from the app's landing site on every supported language).
    for (locale_prefix, path_prefix) in [("", ""), ("/ru", "/ru"), ("/ko", "/ko"), ("/vi", "/vi")] {
        let uri = if locale_prefix.is_empty() {
            "/".to_string()
        } else {
            locale_prefix.to_string()
        };
        let body = get_body(&uri).await;
        let expected_privacy = format!(r#"href="{path_prefix}/privacy""#);
        let expected_terms = format!(r#"href="{path_prefix}/terms""#);
        assert!(
            body.contains(&expected_privacy),
            "footer must link to {expected_privacy} on {uri}"
        );
        assert!(
            body.contains(&expected_terms),
            "footer must link to {expected_terms} on {uri}"
        );
    }
}

#[tokio::test]
async fn privacy_page_renders_breadcrumb_and_h1() {
    let body = get_body("/privacy").await;
    let json = find_jsonld_block_by_type(&body, "BreadcrumbList");
    assert!(
        json.contains("/privacy"),
        "BreadcrumbList for /privacy must reference the page URL: {json}"
    );
    assert!(
        body.contains("legal-doc__title") && body.contains("Privacy Policy"),
        "/privacy must render an h1 with the page title"
    );
}

#[tokio::test]
async fn terms_page_renders_breadcrumb_and_h1() {
    let body = get_body("/terms").await;
    let json = find_jsonld_block_by_type(&body, "BreadcrumbList");
    assert!(
        json.contains("/terms"),
        "BreadcrumbList for /terms must reference the page URL: {json}"
    );
    assert!(
        body.contains("legal-doc__title") && body.contains("Terms of Service"),
        "/terms must render an h1 with the page title"
    );
}
