//! Integration tests for `/blog` index and `/blog/<slug>` article pages.
//!
//! Mirrors the convention in `tests/seo_meta.rs`: render the production
//! router via `tower::ServiceExt::oneshot`, parse the resulting HTML with
//! plain string assertions. Coverage spans all 4 locales (EN/RU/KO/VI) for
//! 2 articles, plus the index page and the legacy-slug 308 redirect.

#![cfg(feature = "ssr")]

use http::StatusCode;

use common::get;

mod common;

// =========================================================================
// Article pages — native rendering across all 4 locales
// =========================================================================

#[tokio::test]
async fn en_article_returns_200_with_h1() {
    let (status, body) = get("/blog/anki-alternative-japanese").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("blog-post__title"),
        "EN article must render inside .blog-post; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    assert!(
        body.contains("Anki Alternative for Japanese"),
        "EN article must contain its title"
    );
}

#[tokio::test]
async fn ru_article_returns_200_with_localized_h1() {
    let (status, body) = get("/ru/blog/best-japanese-learning-app").await;
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
async fn ko_article_returns_200_with_localized_h1() {
    let (status, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("blog-post__title"),
        "KO article must render inside .blog-post; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    assert!(
        body.contains("Anki 대안"),
        "KO article must contain its Korean title fragment; got first 800 chars: {}",
        body.chars().take(800).collect::<String>()
    );
}

#[tokio::test]
async fn vi_article_returns_200_with_localized_h1() {
    let (status, body) = get("/vi/blog/best-japanese-learning-app").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("blog-post__title"),
        "VI article must render inside .blog-post"
    );
    assert!(
        body.contains("ứng dụng học tiếng Nhật"),
        "VI article must contain its Vietnamese title fragment; got first 800 chars: {}",
        body.chars().take(800).collect::<String>()
    );
}

#[tokio::test]
async fn all_eight_article_urls_return_200() {
    // All translations of both articles must be reachable in their native
    // locale. A failure here usually means a markdown file is missing or
    // the ARTICLES const in registry.rs is out of sync with the files on
    // disk.
    let urls = [
        "/blog/anki-alternative-japanese",
        "/ru/blog/anki-alternative-japanese",
        "/ko/blog/anki-alternative-japanese",
        "/vi/blog/anki-alternative-japanese",
        "/blog/best-japanese-learning-app",
        "/ru/blog/best-japanese-learning-app",
        "/ko/blog/best-japanese-learning-app",
        "/vi/blog/best-japanese-learning-app",
    ];
    for uri in urls {
        let (status, _) = get(uri).await;
        assert_eq!(status, StatusCode::OK, "expected 200 for {uri}");
    }
}

// =========================================================================
// 404 / unknown slug
// =========================================================================

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

// =========================================================================
// Sanitization
// =========================================================================

#[tokio::test]
async fn seed_article_html_is_sanitized() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
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

// =========================================================================
// hreflang — now uniform 5-element set on every article (en/ru/ko/vi + x-default)
// =========================================================================

#[tokio::test]
async fn en_article_hreflang_lists_all_4_locales() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    for locale in ["en", "ru", "ko", "vi"] {
        let needle = format!(r#"hreflang="{locale}" href="https://"#);
        assert!(
            body.contains(&needle),
            "EN article must declare hreflang={locale}"
        );
    }
    assert!(
        body.contains(r#"hreflang="x-default""#),
        "EN article must declare hreflang=x-default"
    );
}

#[tokio::test]
async fn ko_article_hreflang_includes_all_4_locales() {
    // After Slice-2 the KO translation exists, so /ko/blog/<slug> is a
    // native render (no fallback). Every translation must be advertised.
    let (_, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"hreflang="ko" href="https://"#),
        "KO native article must self-declare hreflang=ko"
    );
    assert!(
        body.contains(r#"hreflang="en" href="https://"#),
        "KO article must advertise hreflang=en"
    );
    assert!(
        body.contains(r#"hreflang="ru" href="https://"#),
        "KO article must advertise hreflang=ru"
    );
}

#[tokio::test]
async fn ko_article_canonical_points_at_ko_url() {
    let (_, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"/ko/blog/anki-alternative-japanese"#),
        "KO native canonical must reference its own KO URL; got first 2000 chars: {}",
        body.chars().take(2000).collect::<String>()
    );
    assert!(
        !body.contains(r#"name="robots" content="noindex"#),
        "KO native article must NOT carry noindex (it's a real translation)"
    );
}

// =========================================================================
// Script consistency — guards against LLM translation artifacts
// =========================================================================

#[tokio::test]
async fn ko_articles_do_not_contain_cyrillic() {
    // KO translations must not contain Cyrillic characters (hallucinated
    // script mixing is a common LLM failure mode). Hangul + ASCII + CJK
    // for kanji terms are the expected scripts.
    for slug in ["anki-alternative-japanese", "best-japanese-learning-app"] {
        let (_, body) = get(&format!("/ko/blog/{slug}")).await;
        let body_start = body
            .find("<div class=\"blog-post__body\"")
            .unwrap_or(body.len());
        let body_end = body[body_start..]
            .find("</article>")
            .map(|offset| body_start + offset)
            .unwrap_or(body.len());
        let article_html = &body[body_start..body_end];
        let cyrillic_count = article_html
            .chars()
            .filter(|c| ('\u{0400}'..='\u{04FF}').contains(c))
            .count();
        assert_eq!(
            cyrillic_count, 0,
            "KO article {slug} must not contain Cyrillic characters; found {cyrillic_count}"
        );
    }
}

#[tokio::test]
async fn vi_articles_do_not_contain_cyrillic() {
    for slug in ["anki-alternative-japanese", "best-japanese-learning-app"] {
        let (_, body) = get(&format!("/vi/blog/{slug}")).await;
        let body_start = body
            .find("<div class=\"blog-post__body\"")
            .unwrap_or(body.len());
        let body_end = body[body_start..]
            .find("</article>")
            .map(|offset| body_start + offset)
            .unwrap_or(body.len());
        let article_html = &body[body_start..body_end];
        let cyrillic_count = article_html
            .chars()
            .filter(|c| ('\u{0400}'..='\u{04FF}').contains(c))
            .count();
        assert_eq!(
            cyrillic_count, 0,
            "VI article {slug} must not contain Cyrillic characters; found {cyrillic_count}"
        );
    }
}

#[tokio::test]
async fn vi_articles_do_not_contain_korean() {
    // Hangul syllables: U+AC00–U+D7AF. Hallucinated Korean in a VI
    // translation would be a clear LLM error.
    for slug in ["anki-alternative-japanese", "best-japanese-learning-app"] {
        let (_, body) = get(&format!("/vi/blog/{slug}")).await;
        let body_start = body
            .find("<div class=\"blog-post__body\"")
            .unwrap_or(body.len());
        let body_end = body[body_start..]
            .find("</article>")
            .map(|offset| body_start + offset)
            .unwrap_or(body.len());
        let article_html = &body[body_start..body_end];
        let hangul_count = article_html
            .chars()
            .filter(|c| ('\u{AC00}'..='\u{D7AF}').contains(c))
            .count();
        assert_eq!(
            hangul_count, 0,
            "VI article {slug} must not contain Hangul; found {hangul_count}"
        );
    }
}

#[tokio::test]
async fn ru_articles_do_not_contain_korean() {
    for slug in ["anki-alternative-japanese", "best-japanese-learning-app"] {
        let (_, body) = get(&format!("/ru/blog/{slug}")).await;
        let body_start = body
            .find("<div class=\"blog-post__body\"")
            .unwrap_or(body.len());
        let body_end = body[body_start..]
            .find("</article>")
            .map(|offset| body_start + offset)
            .unwrap_or(body.len());
        let article_html = &body[body_start..body_end];
        let hangul_count = article_html
            .chars()
            .filter(|c| ('\u{AC00}'..='\u{D7AF}').contains(c))
            .count();
        assert_eq!(
            hangul_count, 0,
            "RU article {slug} must not contain Hangul; found {hangul_count}"
        );
    }
}

// =========================================================================
// Breadcrumb — 3 levels after the hub landing was added
// =========================================================================

#[tokio::test]
async fn en_article_breadcrumb_has_3_levels() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    let block_start = body
        .find(r#""@type":"BreadcrumbList""#)
        .expect("BreadcrumbList must be present");
    let window = &body[block_start..];
    let window_end = window.find("</script>").unwrap_or(window.len());
    let block = &window[..window_end];
    let positions = block.matches(r#""position":"#).count();
    assert_eq!(
        positions, 3,
        "article breadcrumb must have 3 positions (Home → Blog → Article); got block: {block}"
    );
}

// =========================================================================
// Canonical, JSON-LD, keywords
// =========================================================================

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
    assert!(
        body.contains(r#""@type":"Article""#),
        "blog pages must emit Article JSON-LD; got first 2000 chars: {}",
        body.chars().take(2000).collect::<String>()
    );
}

#[tokio::test]
async fn en_article_has_breadcrumb_jsonld() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#""@type":"BreadcrumbList""#),
        "blog pages must emit BreadcrumbList JSON-LD"
    );
}

#[tokio::test]
async fn en_article_has_keywords_meta_from_frontmatter() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains("anki alternative japanese"),
        "blog keywords meta must come from the article frontmatter"
    );
}

#[tokio::test]
async fn ru_article_has_inline_competitor_citation() {
    let (_, body) = get("/ru/blog/best-japanese-learning-app").await;
    assert!(
        body.contains("wanikani.com"),
        "RU article must link to wanikani.com (competitor citation)"
    );
}

#[tokio::test]
async fn ru_article_does_not_contain_known_typo() {
    let (_, body) = get("/ru/blog/best-japanese-learning-app").await;
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
async fn ru_article_has_ru_og_locale() {
    let (_, body) = get("/ru/blog/best-japanese-learning-app").await;
    assert!(
        body.contains(r#"property="og:locale" content="ru_RU""#),
        "RU article must declare og:locale=ru_RU"
    );
}

#[tokio::test]
async fn en_article_has_en_og_locale() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"property="og:locale" content="en_US""#),
        "EN article must declare og:locale=en_US"
    );
}

#[tokio::test]
async fn ko_article_has_ko_og_locale() {
    let (_, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"property="og:locale" content="ko_KR""#),
        "KO native article must declare og:locale=ko_KR; got first 1500 chars: {}",
        body.chars().take(1500).collect::<String>()
    );
}

#[tokio::test]
async fn vi_article_has_vi_og_locale() {
    let (_, body) = get("/vi/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"property="og:locale" content="vi_VN""#),
        "VI native article must declare og:locale=vi_VN"
    );
}

// =========================================================================
// Blog index page (/blog) — list of articles
// =========================================================================

#[tokio::test]
async fn en_blog_index_returns_200_with_article_list() {
    let (status, body) = get("/blog").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("blog-index"),
        "EN blog index must render .blog-index; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    // EN has both articles; the index must list them.
    assert!(
        body.contains("Anki Alternative for Japanese"),
        "EN blog index must list the Anki article"
    );
    assert!(
        body.contains("Best Japanese Learning App"),
        "EN blog index must list the Best-app article"
    );
}

#[tokio::test]
async fn ru_blog_index_returns_200_with_localized_articles() {
    let (status, body) = get("/ru/blog").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("blog-index"),
        "RU blog index must render .blog-index"
    );
    // Strict filter: RU index shows only RU articles. KO/VI article URLs
    // must NOT appear (would mislead crawlers into thinking /ko/... is RU).
    assert!(
        !body.contains(r#"href="/ko/blog/"#),
        "RU blog index must not link to KO URLs (strict locale filter)"
    );
}

#[tokio::test]
async fn blog_index_has_itemlist_jsonld() {
    let (_, body) = get("/blog").await;
    assert!(
        body.contains(r#""@type":"ItemList""#),
        "blog index must emit ItemList JSON-LD (eligible for Carousel rich result per ADR-018)"
    );
}

#[tokio::test]
async fn blog_index_does_not_shadow_article_route() {
    // Smoke-test on Leptos 0.8 router: static `Route path="blog"` must
    // match `/blog` exactly, and `Route path="blog/:slug"` must still
    // match `/blog/<slug>`. If the static route shadowed the parametric,
    // `/blog/anki-alternative-japanese` would render the index (or 404)
    // instead of the article.
    let (status, body) = get("/blog/anki-alternative-japanese").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("blog-post__title"),
        "article route must still match (router shadow regression)"
    );
    assert!(
        !body.contains("blog-index__list"),
        "article route must not render the blog index page"
    );
}

#[tokio::test]
async fn all_four_blog_index_urls_return_200() {
    for uri in ["/blog", "/ru/blog", "/ko/blog", "/vi/blog"] {
        let (status, body) = get(uri).await;
        assert_eq!(status, StatusCode::OK, "expected 200 for {uri}");
        assert!(
            body.contains("blog-index"),
            "{uri} must render the blog index page"
        );
    }
}

// =========================================================================
// Header navigation — "Blog" link visible across all locales
// =========================================================================

#[tokio::test]
async fn header_includes_blog_link_on_every_locale() {
    let cases = [
        ("/", "/blog", "Blog"),
        ("/ru", "/ru/blog", "Блог"),
        ("/ko", "/ko/blog", "블로그"),
        ("/vi", "/vi/blog", "Blog"),
    ];
    for (uri, expected_href, expected_label) in cases {
        let (_, body) = get(uri).await;
        let expected_attr = format!(r#"href="{expected_href}""#);
        assert!(
            body.contains(&expected_attr),
            "{uri} header must link to {expected_href}"
        );
        assert!(
            body.contains(expected_label),
            "{uri} header must render the localized Blog label `{expected_label}`"
        );
    }
}

// =========================================================================
// Legacy slug 308 redirect
// =========================================================================

#[tokio::test]
async fn old_ru_article_slug_redirects_308() {
    // The RU article was renamed from `luchshee-prilozhenie-...` to the
    // unified `best-japanese-learning-app`. The old URL must 308-redirect
    // to preserve link equity (Google treats 308 as permanent).
    let (status, _body) = get("/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    assert_eq!(
        status.as_u16(),
        308,
        "old RU slug must return 308 Permanent Redirect"
    );
}
