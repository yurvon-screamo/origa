//! Integration tests for `/docs` index and `/docs/<slug>` article pages.
//!
//! Mirrors the convention in `tests/blog.rs`: render the production router
//! via `tower::ServiceExt::oneshot`, parse the resulting HTML with plain
//! string assertions. Docs ship in all four locales (EN/RU/KO/VI), so the
//! KO/VI pages must render natively: localized titles, self-canonical URLs,
//! indexable robots policy, and the full hreflang set.

#![cfg(feature = "ssr")]

use http::StatusCode;

use common::get;

mod common;

/// Compile-time landing base URL — canonical/hreflang assertions stay exact
/// without hardcoding the production domain (survives builds with a custom
/// `ORIGA_LANDING_BASE_URL`).
const BASE_URL: &str = env!("ORIGA_LANDING_BASE_URL");

/// Canonical list of every sidebar doc slug (the `index` page is the docs
/// landing, not an article). Shared by every test below so adding a doc page
/// is a one-line edit here.
const ALL_SLUGS: &[&str] = &[
    "getting-started",
    "lesson",
    "fsrs",
    "vocabulary",
    "kanji",
    "grammar",
    "phrases",
    "capture",
    "limitations",
    "data-sources",
];

// =========================================================================
// Native rendering in KO/VI
// =========================================================================

#[tokio::test]
async fn ko_article_returns_200_with_localized_h1() {
    let (status, body) = get("/ko/docs/fsrs").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"<article class="docs-article">"#),
        "KO doc must render inside .docs-article; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    assert!(
        body.contains("Origa가 무엇을 보여줄지 정하는 방식"),
        "KO doc must contain its Korean H1"
    );
}

#[tokio::test]
async fn vi_article_returns_200_with_localized_h1() {
    let (status, body) = get("/vi/docs/fsrs").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"<article class="docs-article">"#),
        "VI doc must render inside .docs-article; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    assert!(
        body.contains("Origa quyết định thế nào thứ sẽ hiển thị"),
        "VI doc must contain its Vietnamese H1"
    );
}

#[tokio::test]
async fn ko_docs_index_returns_200() {
    let (status, body) = get("/ko/docs").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"<article class="docs-article">"#),
        "KO docs index must render the docs layout"
    );
    assert!(
        body.contains("Origa 문서"),
        "KO docs index must render the Korean index title"
    );
}

#[tokio::test]
async fn vi_docs_index_returns_200() {
    let (status, body) = get("/vi/docs").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("Tài liệu Origa"),
        "VI docs index must render the Vietnamese index title"
    );
}

// =========================================================================
// SEO: native pages are indexable, self-canonical, fully alternated
// =========================================================================

#[tokio::test]
async fn ko_and_vi_docs_are_indexable() {
    // Native pages must NOT carry the fallback robots policy.
    for path in ["/ko/docs/fsrs", "/vi/docs/fsrs", "/ko/docs", "/vi/docs"] {
        let (_, body) = get(path).await;
        assert!(
            !body.contains(r#"name="robots" content="noindex"#),
            "native doc page {path} must not be noindex"
        );
    }
}

#[tokio::test]
async fn ko_article_canonical_points_at_ko_url() {
    let (_, body) = get("/ko/docs/fsrs").await;
    assert!(
        body.contains(&format!(
            r#"rel="canonical" href="{BASE_URL}/ko/docs/fsrs""#
        )),
        "KO native doc must be self-canonical"
    );
}

#[tokio::test]
async fn vi_article_canonical_points_at_vi_url() {
    let (_, body) = get("/vi/docs/fsrs").await;
    assert!(
        body.contains(&format!(
            r#"rel="canonical" href="{BASE_URL}/vi/docs/fsrs""#
        )),
        "VI native doc must be self-canonical"
    );
}

#[tokio::test]
async fn ko_article_hreflang_lists_all_4_locales() {
    let (_, body) = get("/ko/docs/fsrs").await;
    for (lang, prefix) in [("en", ""), ("ru", "/ru"), ("ko", "/ko"), ("vi", "/vi")] {
        assert!(
            body.contains(&format!(
                r#"hreflang="{lang}" href="{BASE_URL}{prefix}/docs/fsrs""#
            )),
            "KO native doc must declare hreflang={lang}"
        );
    }
}

#[tokio::test]
async fn ko_doc_carries_ko_og_locale_and_keywords() {
    let (_, body) = get("/ko/docs/kanji").await;
    assert!(
        body.contains(r#"property="og:locale" content="ko_KR""#),
        "KO doc must declare og:locale=ko_KR"
    );
    assert!(
        body.contains(r#"name="keywords" content="일본어 한자 앱"#),
        "KO doc must carry its localized keywords meta"
    );
}

#[tokio::test]
async fn vi_doc_carries_vi_og_locale_and_keywords() {
    let (_, body) = get("/vi/docs/kanji").await;
    assert!(
        body.contains(r#"property="og:locale" content="vi_VN""#),
        "VI doc must declare og:locale=vi_VN"
    );
    assert!(
        body.contains(r#"name="keywords" content="ứng dụng hán tự tiếng nhật"#),
        "VI doc must carry its localized keywords meta"
    );
}

#[tokio::test]
async fn ko_docs_index_has_item_list_schema() {
    // build_item_list_items fills the ItemList from native pages; with the
    // KO translations shipped it must produce native titles.
    let (_, body) = get("/ko/docs").await;
    assert!(
        body.contains("ItemList"),
        "docs index must embed the ItemList schema"
    );
}

// =========================================================================
// Script consistency: no hallucinated scripts in KO/VI translations
// =========================================================================

#[tokio::test]
async fn ko_docs_do_not_contain_cyrillic() {
    for slug in ALL_SLUGS {
        let (_, body) = get(&format!("/ko/docs/{slug}")).await;
        let cyrillic: usize = body
            .chars()
            .filter(|c| ('\u{0400}'..='\u{04FF}').contains(c))
            .count();
        assert_eq!(
            cyrillic, 0,
            "KO doc {slug} must not contain Cyrillic; found {cyrillic}"
        );
    }
}

#[tokio::test]
async fn vi_docs_do_not_contain_cyrillic() {
    for slug in ALL_SLUGS {
        let (_, body) = get(&format!("/vi/docs/{slug}")).await;
        let cyrillic: usize = body
            .chars()
            .filter(|c| ('\u{0400}'..='\u{04FF}').contains(c))
            .count();
        assert_eq!(
            cyrillic, 0,
            "VI doc {slug} must not contain Cyrillic; found {cyrillic}"
        );
    }
}

#[tokio::test]
async fn vi_docs_do_not_contain_korean() {
    // Hangul syllables: U+AC00–U+D7AF. Hallucinated Korean in a VI
    // translation would be a clear LLM error.
    for slug in ALL_SLUGS {
        let (_, body) = get(&format!("/vi/docs/{slug}")).await;
        let body_start = body
            .find("<div class=\"docs-article__body\"")
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
            "VI doc {slug} must not contain Hangul; found {hangul_count}"
        );
    }
}

#[tokio::test]
async fn vi_docs_do_not_contain_kanji() {
    // VI SEO strategy (marketing/strategies/origa-seo.md §6 Market 3)
    // requires "Hán tự" across the VI locale — "kanji" is the search miss
    // called out as the single highest-impact content fix. Mirrors
    // `vi_articles_do_not_contain_kanji` in tests/blog.rs, including the
    // subtraction classes:
    //   - proper nouns ("KanjiVG" in data-sources — product name)
    //   - hrefs of internal links to /vi/docs/kanji (URL slugs are not
    //     translated, so the anchor target contains the substring while the
    //     link text says "Hán tự")
    const PROPER_NOUNS_WITH_KANJI_SUBSTRING: &[&str] = &["kanjivg"];
    const KANJI_DOC_SLUG_HREF: &str = r#"href="/vi/docs/kanji""#;

    for slug in ALL_SLUGS {
        let (_, body) = get(&format!("/vi/docs/{slug}")).await;
        assert!(
            !body.contains(r#"name="robots" content="noindex"#),
            "VI doc {slug} must render natively, not as an EN fallback"
        );
        let body_start = body
            .find("<div class=\"docs-article__body\"")
            .unwrap_or(body.len());
        let body_end = body[body_start..]
            .find("</article>")
            .map(|offset| body_start + offset)
            .unwrap_or(body.len());
        let article_html = &body[body_start..body_end];
        let lower = article_html.to_lowercase();
        let raw_count = lower.matches("kanji").count();
        let proper_noun_count = PROPER_NOUNS_WITH_KANJI_SUBSTRING
            .iter()
            .map(|s| lower.matches(s).count())
            .sum::<usize>();
        let slug_href_count = lower.matches(KANJI_DOC_SLUG_HREF).count();
        let kanji_count = raw_count - proper_noun_count - slug_href_count;
        assert_eq!(
            kanji_count, 0,
            "VI doc {slug} must use 'hán tự' instead of 'kanji' per SEO strategy; \
             found {kanji_count} conceptual occurrence(s) \
             (raw {raw_count} - proper-noun {proper_noun_count} - slug-href {slug_href_count})"
        );
    }
}
