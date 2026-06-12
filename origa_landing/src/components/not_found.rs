use leptos::prelude::*;

use crate::content::Locale;

#[component]
pub fn NotFound() -> impl IntoView {
    let locale = use_context::<Locale>();
    let text = match locale {
        Some(Locale::Ru) => "Страница не найдена",
        Some(Locale::Ko) => "페이지를 찾을 수 없습니다",
        Some(Locale::Vi) => "Không tìm thấy trang",
        _ => "Page not found",
    };

    view! {
        <div class="landing-hero">
            <h1 class="landing-hero__title">"404"</h1>
            <p class="landing-hero__subtitle">{text}</p>
        </div>
    }
}
