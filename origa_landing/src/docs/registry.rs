//! Static registry of documentation pages, populated on first access from
//! compile-time-embedded markdown sources.
//!
//! Mirrors [`crate::blog::registry`] in structure but covers the `/docs`
//! section. Docs ship in all four locales (EN, RU, KO, VI); the EN fallback
//! in [`crate::pages::docs`] stays as graceful degradation for future pages
//! that land in fewer locales first (the blog already relies on that pattern
//! for its EN+RU cluster). The frontmatter parser
//! ([`crate::blog::frontmatter`]) and markdown renderer
//! ([`crate::blog::render`]) are reused verbatim — the frontmatter shape is
//! identical.

use std::sync::OnceLock;

use crate::blog::frontmatter::{self, ArticleStatus, Frontmatter};
use crate::blog::render::markdown_to_html;
use crate::content::Locale;

/// A single rendered documentation page. The `html` field is sanitized at
/// construction time and safe to inject via `inner_html`.
#[derive(Debug, Clone)]
pub struct DocPage {
    pub slug: &'static str,
    pub locale: Locale,
    pub frontmatter: Frontmatter,
    pub html: String,
}

/// Compile-time manifest of doc pages to ship. Each entry is
/// `(locale, slug, raw_markdown_source)`. Add a new doc page by appending a
/// tuple here and dropping the `.md` file under `content/docs/<locale>/`.
const DOCS: &[(Locale, &str, &str)] = &[
    // index — the /docs landing page (rendered by DocsIndexPage, excluded
    // from the sidebar).
    (
        Locale::En,
        "index",
        include_str!("../../content/docs/en/index.md"),
    ),
    (
        Locale::Ru,
        "index",
        include_str!("../../content/docs/ru/index.md"),
    ),
    (
        Locale::Ko,
        "index",
        include_str!("../../content/docs/ko/index.md"),
    ),
    (
        Locale::Vi,
        "index",
        include_str!("../../content/docs/vi/index.md"),
    ),
    // getting-started
    (
        Locale::En,
        "getting-started",
        include_str!("../../content/docs/en/getting-started.md"),
    ),
    (
        Locale::Ru,
        "getting-started",
        include_str!("../../content/docs/ru/getting-started.md"),
    ),
    (
        Locale::Ko,
        "getting-started",
        include_str!("../../content/docs/ko/getting-started.md"),
    ),
    (
        Locale::Vi,
        "getting-started",
        include_str!("../../content/docs/vi/getting-started.md"),
    ),
    // lesson
    (
        Locale::En,
        "lesson",
        include_str!("../../content/docs/en/lesson.md"),
    ),
    (
        Locale::Ru,
        "lesson",
        include_str!("../../content/docs/ru/lesson.md"),
    ),
    (
        Locale::Ko,
        "lesson",
        include_str!("../../content/docs/ko/lesson.md"),
    ),
    (
        Locale::Vi,
        "lesson",
        include_str!("../../content/docs/vi/lesson.md"),
    ),
    // counters
    (
        Locale::En,
        "counters",
        include_str!("../../content/docs/en/counters.md"),
    ),
    (
        Locale::Ru,
        "counters",
        include_str!("../../content/docs/ru/counters.md"),
    ),
    (
        Locale::Ko,
        "counters",
        include_str!("../../content/docs/ko/counters.md"),
    ),
    (
        Locale::Vi,
        "counters",
        include_str!("../../content/docs/vi/counters.md"),
    ),
    // fsrs
    (
        Locale::En,
        "fsrs",
        include_str!("../../content/docs/en/fsrs.md"),
    ),
    (
        Locale::Ru,
        "fsrs",
        include_str!("../../content/docs/ru/fsrs.md"),
    ),
    (
        Locale::Ko,
        "fsrs",
        include_str!("../../content/docs/ko/fsrs.md"),
    ),
    (
        Locale::Vi,
        "fsrs",
        include_str!("../../content/docs/vi/fsrs.md"),
    ),
    // vocabulary
    (
        Locale::En,
        "vocabulary",
        include_str!("../../content/docs/en/vocabulary.md"),
    ),
    (
        Locale::Ru,
        "vocabulary",
        include_str!("../../content/docs/ru/vocabulary.md"),
    ),
    (
        Locale::Ko,
        "vocabulary",
        include_str!("../../content/docs/ko/vocabulary.md"),
    ),
    (
        Locale::Vi,
        "vocabulary",
        include_str!("../../content/docs/vi/vocabulary.md"),
    ),
    // kanji
    (
        Locale::En,
        "kanji",
        include_str!("../../content/docs/en/kanji.md"),
    ),
    (
        Locale::Ru,
        "kanji",
        include_str!("../../content/docs/ru/kanji.md"),
    ),
    (
        Locale::Ko,
        "kanji",
        include_str!("../../content/docs/ko/kanji.md"),
    ),
    (
        Locale::Vi,
        "kanji",
        include_str!("../../content/docs/vi/kanji.md"),
    ),
    // grammar
    (
        Locale::En,
        "grammar",
        include_str!("../../content/docs/en/grammar.md"),
    ),
    (
        Locale::Ru,
        "grammar",
        include_str!("../../content/docs/ru/grammar.md"),
    ),
    (
        Locale::Ko,
        "grammar",
        include_str!("../../content/docs/ko/grammar.md"),
    ),
    (
        Locale::Vi,
        "grammar",
        include_str!("../../content/docs/vi/grammar.md"),
    ),
    // phrases
    (
        Locale::En,
        "phrases",
        include_str!("../../content/docs/en/phrases.md"),
    ),
    (
        Locale::Ru,
        "phrases",
        include_str!("../../content/docs/ru/phrases.md"),
    ),
    (
        Locale::Ko,
        "phrases",
        include_str!("../../content/docs/ko/phrases.md"),
    ),
    (
        Locale::Vi,
        "phrases",
        include_str!("../../content/docs/vi/phrases.md"),
    ),
    // capture
    (
        Locale::En,
        "capture",
        include_str!("../../content/docs/en/capture.md"),
    ),
    (
        Locale::Ru,
        "capture",
        include_str!("../../content/docs/ru/capture.md"),
    ),
    (
        Locale::Ko,
        "capture",
        include_str!("../../content/docs/ko/capture.md"),
    ),
    (
        Locale::Vi,
        "capture",
        include_str!("../../content/docs/vi/capture.md"),
    ),
    // limitations
    (
        Locale::En,
        "limitations",
        include_str!("../../content/docs/en/limitations.md"),
    ),
    (
        Locale::Ru,
        "limitations",
        include_str!("../../content/docs/ru/limitations.md"),
    ),
    (
        Locale::Ko,
        "limitations",
        include_str!("../../content/docs/ko/limitations.md"),
    ),
    (
        Locale::Vi,
        "limitations",
        include_str!("../../content/docs/vi/limitations.md"),
    ),
    // data-sources
    (
        Locale::En,
        "data-sources",
        include_str!("../../content/docs/en/data-sources.md"),
    ),
    (
        Locale::Ru,
        "data-sources",
        include_str!("../../content/docs/ru/data-sources.md"),
    ),
    (
        Locale::Ko,
        "data-sources",
        include_str!("../../content/docs/ko/data-sources.md"),
    ),
    (
        Locale::Vi,
        "data-sources",
        include_str!("../../content/docs/vi/data-sources.md"),
    ),
];

/// Sidebar navigation order. The `index` page is excluded (it is the docs
/// landing, not a sidebar entry). Every slug listed here must exist in `DOCS`
/// for all four locales — the build_registry assertion guarantees this.
pub const SIDEBAR_SLUGS: &[&str] = &[
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

static REGISTRY: OnceLock<Vec<DocPage>> = OnceLock::new();

/// Build the registry vector by parsing every page in `DOCS`. Called at most
/// once per process via [`OnceLock::get_or_init`]; any later caller reuses
/// the cached value. Panics on a malformed frontmatter or on a draft page —
/// both are programmer errors, not runtime conditions.
fn build_registry() -> Vec<DocPage> {
    DOCS.iter()
        .map(|(locale, slug, src)| {
            let locale_code = locale.as_str();
            let (yaml, body) = frontmatter::split_frontmatter(src)
                .unwrap_or_else(|e| panic!("docs/{locale_code}/{slug}.md: {e}"));
            let fm = frontmatter::parse(yaml)
                .unwrap_or_else(|e| panic!("docs/{locale_code}/{slug}.md: {e}"));
            assert!(
                fm.status == ArticleStatus::Ready,
                "docs/{locale_code}/{slug}.md has status {:?}; only Ready pages may ship",
                fm.status,
            );
            assert_eq!(
                fm.locale, *locale,
                "docs/{locale_code}/{slug}.md frontmatter.locale ({:?}) does not match its directory",
                fm.locale,
            );
            let html = markdown_to_html(body);
            DocPage { slug, locale: *locale, frontmatter: fm, html }
        })
        .collect()
}

/// Read-only access to the full registry. The first call triggers parsing of
/// every page; subsequent calls are free.
pub fn all() -> &'static [DocPage] {
    REGISTRY.get_or_init(build_registry)
}

/// Find a doc page by `(locale, slug)`. Returns `None` if no page exists for
/// that exact pair — callers handling locale fallback (serving the EN page
/// when a future doc ships in fewer locales) must do the fallback themselves.
pub fn find(locale: Locale, slug: &str) -> Option<&'static DocPage> {
    all()
        .iter()
        .find(|page| page.locale == locale && page.slug == slug)
}

/// Locales that have a published translation of `slug`. Used to emit correct
/// `hreflang` alternates: a page should only point at translations that
/// actually exist. Docs ship in all four locales, so this returns a subset of
/// `[En, Ru, Ko, Vi]`.
pub fn locales_for_slug(slug: &str) -> Vec<Locale> {
    all()
        .iter()
        .filter(|page| page.slug == slug)
        .map(|page| page.locale)
        .collect()
}

/// All doc pages published in `locale`, in [`SIDEBAR_SLUGS`] order. Used by
/// the docs index page and the sidebar component. Strict filter — no EN
/// fallback.
pub fn list_by_locale(locale: Locale) -> Vec<&'static DocPage> {
    SIDEBAR_SLUGS
        .iter()
        .filter_map(|slug| find(locale, slug))
        .collect()
}

/// Sidebar entries for `locale` — all pages listed in [`SIDEBAR_SLUGS`] order.
/// Falls back to EN titles when a page is not translated in the requested
/// locale, so the sidebar stays complete for future pages that ship in fewer
/// locales first.
pub fn sidebar_entries(locale: Locale) -> Vec<(&'static str, String, String)> {
    SIDEBAR_SLUGS
        .iter()
        .map(|slug| {
            let page = find(locale, slug)
                .or_else(|| find(Locale::En, slug))
                .expect("SIDEBAR_SLUGS entry must exist in the registry");
            let href = format!("{}/docs/{slug}", locale.path_prefix());
            (*slug, page.frontmatter.title.clone(), href)
        })
        .collect()
}

/// The index page for `locale`, falling back to EN when the locale has no
/// index translation. Used by the DocsIndexPage component.
pub fn index_page(locale: Locale) -> &'static DocPage {
    find(locale, "index")
        .or_else(|| find(Locale::En, "index"))
        .expect("docs index page must exist in EN")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_en_index() {
        let pages = all();
        assert!(
            pages
                .iter()
                .any(|p| p.slug == "index" && p.locale == Locale::En),
            "EN index page missing from registry: {pages:?}"
        );
    }

    #[test]
    fn registry_size_matches_docs_const() {
        assert_eq!(
            all().len(),
            DOCS.len(),
            "registry length must match DOCS const (expected {}, got {})",
            DOCS.len(),
            all().len()
        );
    }

    #[test]
    fn find_returns_en_getting_started() {
        let page = find(Locale::En, "getting-started").expect("page must be findable");
        assert_eq!(page.locale, Locale::En);
        assert!(
            !page.html.is_empty(),
            "page body must render to non-empty HTML"
        );
    }

    #[test]
    fn find_returns_none_for_unknown_slug() {
        assert!(find(Locale::En, "does-not-exist").is_none());
    }

    #[test]
    fn locales_for_slug_returns_all_four_locales() {
        let locales = locales_for_slug("getting-started");
        assert!(locales.contains(&Locale::En), "missing EN: {locales:?}");
        assert!(locales.contains(&Locale::Ru), "missing RU: {locales:?}");
        assert!(locales.contains(&Locale::Ko), "missing KO: {locales:?}");
        assert!(locales.contains(&Locale::Vi), "missing VI: {locales:?}");
        assert_eq!(locales.len(), 4, "docs ship in all 4 locales: {locales:?}");
    }

    #[test]
    fn sidebar_excludes_index_and_has_fixed_order() {
        let entries_en = list_by_locale(Locale::En);
        let slugs: Vec<&str> = entries_en.iter().map(|p| p.slug).collect();
        assert_eq!(
            slugs, SIDEBAR_SLUGS,
            "sidebar must follow SIDEBAR_SLUGS order"
        );
        assert!(
            !slugs.contains(&"index"),
            "index must not appear in sidebar entries"
        );
    }

    #[test]
    fn every_sidebar_slug_exists_in_all_four_locales() {
        for slug in SIDEBAR_SLUGS {
            assert!(find(Locale::En, slug).is_some(), "EN {slug} missing");
            assert!(find(Locale::Ru, slug).is_some(), "RU {slug} missing");
            assert!(find(Locale::Ko, slug).is_some(), "KO {slug} missing");
            assert!(find(Locale::Vi, slug).is_some(), "VI {slug} missing");
        }
    }

    #[test]
    fn sidebar_entries_are_native_for_ko() {
        let entries = sidebar_entries(Locale::Ko);
        assert_eq!(entries.len(), SIDEBAR_SLUGS.len());
        for (slug, _, href) in &entries {
            assert!(
                href.starts_with("/ko/docs/"),
                "KO sidebar entry {slug} must link at the native KO page, got {href}"
            );
        }
    }

    #[test]
    fn ko_pages_render_korean_titles() {
        // The KO translation must actually be served (not the EN fallback):
        // the getting-started title is Korean text.
        let page = find(Locale::Ko, "getting-started").expect("KO page present");
        assert_eq!(page.frontmatter.title, "Origa 시작하기");
    }

    #[test]
    fn page_html_is_sanitized() {
        let page = find(Locale::En, "getting-started").expect("page present");
        assert!(
            !page.html.contains("<script"),
            "rendered page must not contain <script>; got: {}",
            &page.html[..page.html.len().min(400)]
        );
    }
}
