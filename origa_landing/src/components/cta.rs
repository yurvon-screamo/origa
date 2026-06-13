use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub(crate) fn CtaSection(
    title: &'static str,
    button_text: &'static str,
    download_href: String,
) -> impl IntoView {
    view! {
        <section class="home-cta">
            <hr class="home-cta__rule" />
            <h2 class="home-cta__title">{title}</h2>
            <A href=download_href attr:class="btn btn-filled">{button_text}</A>
            <p class="home-cta__platforms">
                "Windows · Linux · macOS · Android · iOS · Web"
            </p>
        </section>
    }
}
