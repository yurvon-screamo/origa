//! `/blog/<slug>` page: renders a single article with article-grade SEO
//! metadata. When the requested locale has no translation, the page falls
//! back to the English article with `robots: noindex, follow` and a
//! canonical URL pointing at the EN version — correct SEO behaviour for
//! untranslated content served from a translated URL prefix.

use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_params_map;

use crate::blog::{self, BlogPost};
use crate::components::seo::{SchemaOrg, article_schema, breadcrumb_schema};
use crate::content::Locale;

const BASE_URL: &str = env!("ORIGA_LANDING_BASE_URL");

/// Static descriptor of how a request was resolved. The fallback variant
/// records that the user asked for a non-EN locale but received the EN
/// article — drives the `noindex` + canonical-redirect SEO signals.
enum Resolution {
    Native(&'static BlogPost),
    Fallback(&'static BlogPost),
    NotFound,
}

#[component]
pub fn BlogPostPage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let params = use_params_map();
    let slug = params.read().get("slug").unwrap_or_default();

    let resolution = resolve(locale, &slug);

    match resolution {
        Resolution::NotFound => crate::components::NotFound.into_any(),
        Resolution::Native(post) => render_article(locale, post, false).into_any(),
        Resolution::Fallback(post) => render_article(locale, post, true).into_any(),
    }
}

fn resolve(locale: Locale, slug: &str) -> Resolution {
    if slug.is_empty() {
        return Resolution::NotFound;
    }
    if let Some(post) = blog::find(locale, slug) {
        return Resolution::Native(post);
    }
    match blog::find(Locale::En, slug) {
        Some(post) => Resolution::Fallback(post),
        None => Resolution::NotFound,
    }
}

/// Computed SEO metadata for one article render. Grouping these into a
/// struct keeps [`render_article`] short and lets unit tests cover URL /
/// hreflang logic without spinning up the full Leptos view.
struct ArticleMeta {
    canonical: String,
    keywords: String,
    og_image: String,
    og_locale: &'static str,
    canonical_locale: Locale,
    x_default_locale: Locale,
}

impl ArticleMeta {
    fn compute(requested_locale: Locale, post: &BlogPost, is_fallback: bool) -> Self {
        let canonical_locale = if is_fallback {
            Locale::En
        } else {
            requested_locale
        };
        let translation_locales = blog::locales_for_slug(post.slug);

        // `x-default` must point at a URL that actually serves content. The
        // EN article is the canonical fallback when it exists; otherwise we
        // reuse the canonical URL of the article being served (e.g. an
        // RU-only post has no EN version to fall back to, so x-default
        // points at itself).
        let x_default_locale = if translation_locales.contains(&Locale::En) {
            Locale::En
        } else {
            canonical_locale
        };

        Self {
            canonical: format!("{BASE_URL}{}", article_url(canonical_locale, post.slug)),
            keywords: post.frontmatter.target_keywords.join(", "),
            og_image: format!(
                "{BASE_URL}/images/{}.hero.png",
                requested_locale.image_prefix()
            ),
            og_locale: if is_fallback {
                Locale::En.og_locale()
            } else {
                requested_locale.og_locale()
            },
            canonical_locale,
            x_default_locale,
        }
    }
}

fn render_article(
    requested_locale: Locale,
    post: &'static BlogPost,
    is_fallback: bool,
) -> impl IntoView {
    let meta = ArticleMeta::compute(requested_locale, post, is_fallback);
    let article_json = article_schema(meta.canonical_locale, post, &meta.canonical);
    let breadcrumb_json = breadcrumb_schema(
        meta.canonical_locale,
        &article_url(meta.canonical_locale, post.slug),
        post.frontmatter.title.as_str(),
    );
    let translation_locales = blog::locales_for_slug(post.slug);

    view! {
        <Title text=post.frontmatter.meta_title.clone()/>
        <Meta name="description" content=post.frontmatter.meta_description.clone()/>
        <Meta name="keywords" content=meta.keywords/>
        {if is_fallback {
            view! { <Meta name="robots" content="noindex, follow"/> }.into_any()
        } else {
            ().into_any()
        }}
        <Meta property="og:title" content=post.frontmatter.meta_title.clone()/>
        <Meta property="og:description" content=post.frontmatter.meta_description.clone()/>
        <Meta property="og:type" content="article"/>
        <Meta property="og:image" content=meta.og_image.clone()/>
        <Meta property="og:url" content=meta.canonical.clone()/>
        <Meta property="og:locale" content=meta.og_locale/>
        <Meta name="twitter:card" content="summary_large_image"/>
        <Meta name="twitter:title" content=post.frontmatter.meta_title.clone()/>
        <Meta name="twitter:description" content=post.frontmatter.meta_description.clone()/>
        <Meta name="twitter:image" content=meta.og_image/>
        <link rel="canonical" href=meta.canonical.clone()/>
        {translation_locales
            .iter()
            .map(|loc| {
                let href = format!("{BASE_URL}{}", article_url(*loc, post.slug));
                view! { <link rel="alternate" hreflang=loc.as_str() href=href.clone()/> }
            })
            .collect_view()}
        <link rel="alternate" hreflang="x-default" href=format!(
            "{BASE_URL}{}",
            article_url(meta.x_default_locale, post.slug),
        )/>

        <SchemaOrg json=article_json/>
        <SchemaOrg json=breadcrumb_json/>

        <ArticleBody requested_locale post is_fallback />
    }
}

#[component]
fn ArticleBody(
    requested_locale: Locale,
    post: &'static BlogPost,
    is_fallback: bool,
) -> impl IntoView {
    let locale_marker = if is_fallback {
        format!(
            "Showing English article · {}",
            requested_locale.display_label()
        )
    } else {
        requested_locale.display_label().to_string()
    };

    view! {
        <article class="blog-post">
            <header class="blog-post__header">
                <p class="blog-post__locale-marker">{locale_marker}</p>
                <h1 class="blog-post__title">{post.frontmatter.title.clone()}</h1>
                <p class="blog-post__updated">
                    {lastmod_label(requested_locale)} " " {post.frontmatter.lastmod.clone()}
                </p>
            </header>
            <div class="blog-post__body" inner_html=post.html.as_str()></div>
        </article>
    }
}

fn article_url(locale: Locale, slug: &str) -> String {
    // `Locale::En.path_prefix()` is `""`, so a single format string covers
    // both the EN (`/blog/<slug>`) and prefixed (`/ru/blog/<slug>`) cases.
    format!("{}/blog/{slug}", locale.path_prefix())
}

fn lastmod_label(locale: Locale) -> &'static str {
    match locale {
        Locale::Ru => "Обновлено:",
        Locale::Ko => "업데이트:",
        Locale::Vi => "Cập nhật:",
        Locale::En => "Updated:",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn article_url_en_has_no_locale_prefix() {
        assert_eq!(
            article_url(Locale::En, "anki-alternative-japanese"),
            "/blog/anki-alternative-japanese"
        );
    }

    #[test]
    fn article_url_ru_has_locale_prefix() {
        assert_eq!(
            article_url(Locale::Ru, "luchshee-prilozhenie"),
            "/ru/blog/luchshee-prilozhenie"
        );
    }
}
