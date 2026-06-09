use leptos::prelude::*;
use leptos_router::components::{A, Outlet};

use crate::content::Locale;

fn make_href(prefix: &str, page: &str) -> String {
    format!("{prefix}/{page}")
}

#[component]
pub fn Layout(locale: Locale) -> impl IntoView {
    provide_context(locale);

    let c = locale.content();
    let prefix = locale.path_prefix();

    let home_href = if prefix.is_empty() {
        "/".to_string()
    } else {
        prefix.to_string()
    };

    let lang_switcher_items: Vec<_> = Locale::ALL
        .iter()
        .filter(|loc| **loc != locale)
        .map(|loc| {
            let href = if loc.path_prefix().is_empty() {
                "/".to_string()
            } else {
                loc.path_prefix().to_string()
            };
            (*loc, loc.display_label(), href)
        })
        .collect();

    view! {
        {match locale {
            Locale::Ru => view! {
                <script>"document.documentElement.lang=\"ru\""</script>
            }
            .into_any(),
            Locale::Ko => view! {
                <script>"document.documentElement.lang=\"ko\""</script>
            }
            .into_any(),
            Locale::Vi => view! {
                <script>"document.documentElement.lang=\"vi\""</script>
            }
            .into_any(),
            Locale::En => ().into_any(),
        }}
        <header class="landing-header">
            <a href=home_href class="landing-header__logo">
                <img src="/images/logo.png" alt="Origa" class="landing-header__logo-img" />
                <span class="landing-header__logo-name">"Origa"</span>
                <span class="landing-header__logo-kana">"オリガ"</span>
            </a>
            <input type="checkbox" id="nav-toggle" class="landing-header__toggle"
                   aria-label="Toggle navigation" />
            <label for="nav-toggle" class="landing-header__hamburger"
                   tabindex="0" aria-label="Open menu">
                <span class="landing-header__hamburger-line"></span>
            </label>
            <nav class="landing-header__nav" id="main-nav"
                 aria-label="Main navigation">
                <NavLink prefix href="features" class="landing-header__link">
                    {c.header_features}
                </NavLink>
                <NavLink prefix href="compare" class="landing-header__link">
                    {c.header_compare}
                </NavLink>
                <NavLink prefix href="content" class="landing-header__link">
                    {c.header_integrations}
                </NavLink>
                <NavLink prefix href="download" class="landing-header__link">
                    {c.header_download}
                </NavLink>
                <span class="landing-header__nav-sep">"|"</span>
                <span class="landing-header__lang">
                    <span class="landing-header__lang-current">{locale.display_label()}</span>
                    {lang_switcher_items.into_iter().flat_map(|(_, label, href)| {
                        vec![
                            view! { <span class="landing-header__lang-sep">" · "</span> }.into_any(),
                            view! { <a href=href class="landing-header__lang-link">{label}</a> }
                                .into_any(),
                        ]
                    }).collect_view()}
                </span>
            </nav>
        </header>
        {if locale.is_development() {
            view! {
                <div class="landing-wip-banner">{c.banner_wip}</div>
            }
                .into_any()
        } else {
            ().into_any()
        }}
        <main>
            <Outlet />
        </main>
        <footer class="landing-footer">
            <div class="landing-footer__grid">
                <div>
                    <p class="landing-footer__heading">{c.footer_product}</p>
                    <NavLink prefix href="features" class="landing-footer__link">
                        {c.header_features}
                    </NavLink>
                    <NavLink prefix href="compare" class="landing-footer__link">
                        {c.header_compare}
                    </NavLink>
                    <NavLink prefix href="content" class="landing-footer__link">
                        {c.header_integrations}
                    </NavLink>
                    <NavLink prefix href="download" class="landing-footer__link">
                        {c.header_download}
                    </NavLink>
                </div>
                <div>
                    <p class="landing-footer__heading">{c.footer_resources}</p>
                    <a
                        href="https://github.com/yurvon-screamo/origa"
                        class="landing-footer__link"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "GitHub"
                    </a>
                    <a
                        href="https://github.com/yurvon-screamo/origa/blob/master/LICENSE"
                        class="landing-footer__link"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        {c.footer_license}
                    </a>
                </div>
            </div>
        </footer>
    }
}

#[component]
fn NavLink(
    prefix: &'static str,
    href: &'static str,
    class: &'static str,
    children: Children,
) -> impl IntoView {
    let target = make_href(prefix, href);
    view! {
        <A href=target attr:class=class>{children()}</A>
    }
}
