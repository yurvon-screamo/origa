//! Integration tests for `/blog/<slug>` SSR pages.
//!
//! Mirrors the convention in `tests/seo_meta.rs`: render the production
//! router via `tower::ServiceExt::oneshot`, parse the resulting HTML with
//! plain string assertions. The seed article (`anki-alternative-japanese`,
//! EN) is exercised across native, fallback, and not-found paths.

#![cfg(feature = "ssr")]

use http::StatusCode;

use common::get;

mod common;

#[tokio::test]
async fn en_seed_article_returns_200_with_h1() {
    let (status, body) = get("/blog/anki-alternative-japanese").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("blog-post__title"),
        "EN seed article must render inside .blog-post; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    assert!(
        body.contains("Anki Alternative for Japanese"),
        "EN seed article must contain its title; got first 800 chars: {}",
        body.chars().take(800).collect::<String>()
    );
}

#[tokio::test]
async fn unknown_slug_returns_404() {
    let (status, body) = get("/blog/this-slug-does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        !body.contains("blog-post__title"),
        "404 must not render a blog article; got: {}",
        body.chars().take(400).collect::<String>()
    );
}

#[tokio::test]
async fn seed_article_html_is_sanitized() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    // Assert on the article body only — the page also emits JSON-LD via
    // `<script type="application/ld+json">`, which is a safe allowlisted
    // script type and would falsely trigger a naive `<script` check.
    let body_start = body
        .find("<div class=\"blog-post__body\"")
        .expect("article body must be present");
    let body_end = body[body_start..]
        .find("</article>")
        .map(|offset| body_start + offset)
        .unwrap_or(body.len());
    let article_html = &body[body_start..body_end];
    assert!(
        !article_html.contains("<script"),
        "rendered article must not contain <script>; got: {article_html}"
    );
    assert!(
        !article_html.contains("onerror="),
        "rendered article must not contain inline event handlers; got: {article_html}"
    );
}

#[tokio::test]
async fn ko_fallback_canonical_points_at_en_url() {
    let (_, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"<link rel="canonical" href="https://"#),
        "KO fallback must have a canonical URL"
    );
    assert!(
        body.contains("/blog/anki-alternative-japanese"),
        "KO fallback canonical must reference the EN article URL (no /ru/ prefix); got first 2000 chars: {}",
        body.chars().take(2000).collect::<String>()
    );
    assert!(
        !body.contains(r#"/ko/blog/anki-alternative-japanese"#),
        "KO fallback canonical must NOT point at the KO URL"
    );
}

#[tokio::test]
async fn en_article_has_canonical_url() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"<link rel="canonical" href="https://"#),
        "blog pages must have a canonical URL; got first 1500 chars: {}",
        body.chars().take(1500).collect::<String>()
    );
    assert!(
        body.contains("/blog/anki-alternative-japanese"),
        "canonical must reference the article slug"
    );
}

#[tokio::test]
async fn en_article_has_article_jsonld() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    let needle = r#""@type":"Article""#;
    assert!(
        body.contains(needle),
        "blog pages must emit Article JSON-LD; got first 2000 chars: {}",
        body.chars().take(2000).collect::<String>()
    );
}

#[tokio::test]
async fn en_article_has_breadcrumb_jsonld() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    let needle = r#""@type":"BreadcrumbList""#;
    assert!(
        body.contains(needle),
        "blog pages must emit BreadcrumbList JSON-LD; got first 2000 chars: {}",
        body.chars().take(2000).collect::<String>()
    );
}

#[tokio::test]
async fn en_article_has_keywords_meta_from_frontmatter() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains("anki alternative japanese"),
        "blog keywords meta must come from the article frontmatter; got first 1500 chars: {}",
        body.chars().take(1500).collect::<String>()
    );
}

#[tokio::test]
async fn ru_article_returns_200_with_localized_h1() {
    let (status, body) = get("/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("blog-post__title"),
        "RU article must render inside .blog-post; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    assert!(
        body.contains("лучшее приложение"),
        "RU article must contain its Russian title"
    );
}

#[tokio::test]
async fn ru_article_does_not_contain_known_typo() {
    // Regression: the original marketing draft had a typo `食ьба` (invalid
    // mix of kanji + Cyrillic soft sign) which slipped through copy-paste.
    // The migrated article must use `食事` instead.
    let (_, body) = get("/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    assert!(
        !body.contains("食ьба"),
        "RU article must not contain the legacy typo `食ьба`"
    );
    assert!(
        body.contains("食事"),
        "RU article must contain the corrected `食事`"
    );
}

#[tokio::test]
async fn ru_article_has_inline_competitor_citation() {
    // The factcheck gate required every competitor mention to carry an
    // inline link to that competitor's official site. This guards against
    // a future editorial pass that strips the citations without re-running
    // the factcheck.
    let (_, body) = get("/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    assert!(
        body.contains("wanikani.com"),
        "RU article must link to wanikani.com (competitor citation)"
    );
}

#[tokio::test]
async fn ru_article_canonical_points_to_ru_url() {
    let (_, body) = get("/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    assert!(
        body.contains(r#"/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo"#),
        "RU article canonical must reference its own RU URL"
    );
}

#[tokio::test]
async fn en_request_for_ru_only_slug_returns_404() {
    // The RU article has no EN translation. The EN URL prefix must not
    // serve the RU article as a silent fallback — EN has no article to
    // show, so the response must be 404.
    let (status, _) = get("/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn en_article_hreflang_lists_only_en_and_x_default() {
    // The seed article exists only in EN; the page must not advertise
    // hreflang alternates for locales that have no translation. Otherwise
    // crawlers discover URLs that fall back to EN (or 404) and waste crawl
    // budget on duplicate/missing content.
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"hreflang="en" href="https://"#),
        "EN article must self-declare hreflang=en; got first 2000 chars: {}",
        body.chars().take(2000).collect::<String>()
    );
    assert!(
        body.contains(r#"hreflang="x-default""#),
        "EN article must declare hreflang=x-default"
    );
    assert!(
        !body.contains(r#"hreflang="ru""#),
        "EN article must not declare hreflang=ru (no RU translation exists)"
    );
    assert!(
        !body.contains(r#"hreflang="ko""#),
        "EN article must not declare hreflang=ko (no KO translation exists)"
    );
    assert!(
        !body.contains(r#"hreflang="vi""#),
        "EN article must not declare hreflang=vi (no VI translation exists)"
    );
}

#[tokio::test]
async fn ru_article_hreflang_lists_only_ru_and_x_default() {
    let (_, body) = get("/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    assert!(
        body.contains(r#"hreflang="ru" href="https://"#),
        "RU article must self-declare hreflang=ru"
    );
    assert!(
        body.contains(r#"hreflang="x-default""#),
        "RU article must declare hreflang=x-default"
    );
    assert!(
        !body.contains(r#"hreflang="en" href="https://"#),
        "RU-only article must not declare hreflang=en (no EN translation exists)"
    );
}

#[tokio::test]
async fn ru_article_x_default_points_to_ru_url_when_no_en_translation() {
    // For an article that exists only in RU, x-default must point at the
    // RU URL (the only serving variant), not at a non-existent EN URL that
    // would 404. This protects crawlers from landing on a dead link when
    // they follow the x-default hint.
    let (_, body) = get("/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    let x_default_idx = body
        .find(r#"hreflang="x-default""#)
        .expect("x-default alternate must be present");
    let window = &body[x_default_idx..];
    assert!(
        window.contains("/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo"),
        "x-default for an RU-only article must point at the RU URL; got: {window}"
    );
}

#[tokio::test]
async fn ru_article_has_ru_og_locale() {
    let (_, body) = get("/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    assert!(
        body.contains(r#"property="og:locale" content="ru_RU""#),
        "RU article must declare og:locale=ru_RU; got first 1500 chars: {}",
        body.chars().take(1500).collect::<String>()
    );
}

#[tokio::test]
async fn en_article_has_en_og_locale() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"property="og:locale" content="en_US""#),
        "EN article must declare og:locale=en_US; got first 1500 chars: {}",
        body.chars().take(1500).collect::<String>()
    );
}

#[tokio::test]
async fn ko_fallback_serves_en_article_with_noindex() {
    // KO has no translation of the seed article. The page must still serve
    // the EN article (better UX than 404), but with `robots: noindex,
    // follow` so Google does not index a KO URL with EN content (duplicate
    // content + hreflang mismatch).
    let (status, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"name="robots" content="noindex, follow""#),
        "KO fallback must carry robots:noindex,follow; got first 1500 chars: {}",
        body.chars().take(1500).collect::<String>()
    );
    assert!(
        body.contains("Anki Alternative for Japanese"),
        "KO fallback must render the EN article content"
    );
}

#[tokio::test]
async fn ko_fallback_og_locale_is_en_us() {
    // The content served is EN even though the URL prefix is /ko. og:locale
    // describes the content language for social-media preview cards, so it
    // must say en_US — saying ko_KR would mislabel EN copy as Korean.
    let (_, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"property="og:locale" content="en_US""#),
        "KO fallback must declare og:locale=en_US (content language); got first 1500 chars: {}",
        body.chars().take(1500).collect::<String>()
    );
}

#[tokio::test]
async fn vi_fallback_serves_en_article_with_noindex() {
    let (status, body) = get("/vi/blog/anki-alternative-japanese").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"name="robots" content="noindex, follow""#),
        "VI fallback must carry robots:noindex,follow"
    );
}

#[tokio::test]
async fn ko_fallback_for_ru_only_slug_returns_404() {
    // The RU article has no EN translation either, so /ko/blog/<ru-slug>
    // cannot fall back to EN. The correct response is 404, not a silent
    // 200 with placeholder content.
    let (status, _) = get("/ko/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn ko_fallback_hreflang_omits_ko_and_vi() {
    let (_, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"hreflang="en" href="https://"#),
        "KO fallback must self-declare hreflang=en (content locale)"
    );
    assert!(
        !body.contains(r#"hreflang="ko""#),
        "KO fallback must NOT declare hreflang=ko (no KO content exists)"
    );
    assert!(
        !body.contains(r#"hreflang="vi""#),
        "KO fallback must NOT declare hreflang=vi"
    );
}
