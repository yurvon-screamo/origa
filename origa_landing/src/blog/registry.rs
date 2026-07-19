//! Static registry of blog articles, populated on first access from
//! compile-time-embedded markdown sources.
//!
//! Each article is a tuple of `(locale, slug, source)`. The registry parses
//! every article on first access via [`OnceLock::get_or_init`], panicking on
//! a malformed frontmatter or on a `status: draft` article. The
//! panic-on-draft invariant is what makes "drafts don't ship" a build-time
//! guarantee: any `cargo test --workspace` run that touches the registry
//! will surface the draft before it reaches production.

use std::sync::OnceLock;

use crate::blog::frontmatter::{self, ArticleStatus, Frontmatter};
use crate::blog::render::markdown_to_html;
use crate::content::Locale;

/// A single rendered blog article. The `html` field is sanitized at
/// construction time and safe to inject via `inner_html`.
#[derive(Debug, Clone)]
pub struct BlogPost {
    pub slug: &'static str,
    pub locale: Locale,
    pub frontmatter: Frontmatter,
    pub html: String,
}

/// Compile-time manifest of articles to ship. Each entry is
/// `(locale, slug, raw_markdown_source)`. Add a new article by appending a
/// tuple here and dropping the `.md` file under `content/blog/<locale>/`.
const ARTICLES: &[(Locale, &str, &str)] = &[
    (
        Locale::En,
        "anki-alternative-japanese",
        include_str!("../../content/blog/en/anki-alternative-japanese.md"),
    ),
    (
        Locale::Ru,
        "luchshee-prilozhenie-izucheniya-yaponskogo",
        include_str!("../../content/blog/ru/luchshee-prilozhenie-izucheniya-yaponskogo.md"),
    ),
];

static REGISTRY: OnceLock<Vec<BlogPost>> = OnceLock::new();

/// Build the registry vector by parsing every article in `ARTICLES`. Called
/// at most once per process via [`OnceLock::get_or_init`]; any later caller
/// reuses the cached value. Panics on a malformed frontmatter or on a draft
/// article — both are programmer errors, not runtime conditions.
fn build_registry() -> Vec<BlogPost> {
    ARTICLES
        .iter()
        .map(|(locale, slug, src)| {
            let locale_code = locale.as_str();
            let (yaml, body) = frontmatter::split_frontmatter(src)
                .unwrap_or_else(|e| panic!("blog/{locale_code}/{slug}.md: {e}"));
            let fm = frontmatter::parse(yaml)
                .unwrap_or_else(|e| panic!("blog/{locale_code}/{slug}.md: {e}"));
            assert!(
                fm.status == ArticleStatus::Ready,
                "blog/{locale_code}/{slug}.md has status {:?}; only Ready articles may ship",
                fm.status,
            );
            assert_eq!(
                fm.locale, *locale,
                "blog/{locale_code}/{slug}.md frontmatter.locale ({:?}) does not match its directory",
                fm.locale,
            );
            let html = markdown_to_html(body);
            BlogPost { slug, locale: *locale, frontmatter: fm, html }
        })
        .collect()
}

/// Read-only access to the full registry. The first call triggers parsing
/// of every article; subsequent calls are free.
pub fn all() -> &'static [BlogPost] {
    REGISTRY.get_or_init(build_registry)
}

/// Find an article by `(locale, slug)`. Returns `None` if no article exists
/// for that exact pair — callers handling locale fallback (e.g. serving the
/// EN article on `/ko/blog/<slug>`) must do the fallback themselves.
pub fn find(locale: Locale, slug: &str) -> Option<&'static BlogPost> {
    all()
        .iter()
        .find(|post| post.locale == locale && post.slug == slug)
}

/// Locales that have a published translation of `slug`. Used to emit correct
/// `hreflang` alternates: a page should only point at translations that
/// actually exist, never at URLs that would 404 or fall back.
pub fn locales_for_slug(slug: &str) -> Vec<Locale> {
    all()
        .iter()
        .filter(|post| post.slug == slug)
        .map(|post| post.locale)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_expected_seed_article() {
        let posts = all();
        assert!(
            posts
                .iter()
                .any(|p| p.slug == "anki-alternative-japanese" && p.locale == Locale::En),
            "seed EN article missing from registry: {posts:?}"
        );
    }

    #[test]
    fn registry_contains_ru_article() {
        let posts = all();
        assert!(
            posts
                .iter()
                .any(|p| p.slug == "luchshee-prilozhenie-izucheniya-yaponskogo"
                    && p.locale == Locale::Ru),
            "RU article missing from registry: {posts:?}"
        );
    }

    #[test]
    fn find_returns_seed_article() {
        let post =
            find(Locale::En, "anki-alternative-japanese").expect("seed article must be findable");
        assert_eq!(post.locale, Locale::En);
        assert!(
            !post.html.is_empty(),
            "article body must render to non-empty HTML"
        );
    }

    #[test]
    fn find_returns_none_for_unknown_slug() {
        assert!(find(Locale::En, "does-not-exist").is_none());
    }

    #[test]
    fn find_returns_none_when_locale_missing() {
        // No RU translation of the EN seed exists yet; the lookup must
        // return None rather than silently returning the EN article.
        // Fallback is the caller's responsibility
        // (see `pages::blog::BlogPostPage`).
        assert!(find(Locale::Ru, "anki-alternative-japanese").is_none());
    }

    #[test]
    fn locales_for_seed_slug_returns_only_en() {
        let locales = locales_for_slug("anki-alternative-japanese");
        assert_eq!(locales, vec![Locale::En]);
    }

    #[test]
    fn locales_for_ru_slug_returns_only_ru() {
        let locales = locales_for_slug("luchshee-prilozhenie-izucheniya-yaponskogo");
        assert_eq!(locales, vec![Locale::Ru]);
    }

    #[test]
    fn seed_article_html_is_sanitized() {
        let post = find(Locale::En, "anki-alternative-japanese").expect("seed present");
        assert!(
            !post.html.contains("<script"),
            "rendered article must not contain <script>; got: {}",
            &post.html[..post.html.len().min(400)]
        );
    }
}
