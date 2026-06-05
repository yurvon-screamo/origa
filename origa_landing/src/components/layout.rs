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

    let switch_locale = match locale {
        Locale::En => Locale::Ru,
        Locale::Ru => Locale::En,
    };
    let switch_label = switch_locale.as_str().to_uppercase();
    let switch_href = match switch_locale {
        Locale::En => "/".to_string(),
        Locale::Ru => "/ru".to_string(),
    };

    view! {
        {match locale {
            Locale::Ru => view! {
                <script>"document.documentElement.lang=\"ru\""</script>
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
            <nav class="landing-header__nav">
                <NavLink prefix href="features" class="landing-header__link">
                    {c.header_features}
                </NavLink>
                <NavLink prefix href="compare" class="landing-header__link">
                    {c.header_compare}
                </NavLink>
                <NavLink prefix href="download" class="landing-header__link">
                    {c.header_download}
                </NavLink>
                <a href=switch_href class="landing-header__link">
                    {switch_label}
                </a>
            </nav>
        </header>
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
