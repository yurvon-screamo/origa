use leptos::prelude::*;

use crate::content::Locale;

#[component]
pub fn NotFound() -> impl IntoView {
    let locale = use_context::<Locale>();
    let text = match locale {
        Some(Locale::Ru) => "Страница не найдена",
        _ => "Page not found",
    };

    view! {
        <div class="landing-hero">
            <h1 class="landing-hero__title">"404"</h1>
            <p class="landing-hero__subtitle">{text}</p>
        </div>
    }
}
