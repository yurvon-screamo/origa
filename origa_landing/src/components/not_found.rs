use leptos::prelude::*;

use crate::content::Locale;

/// 404 page rendered for unmatched routes.
///
/// Sets the HTTP status to `404 Not Found` on the server so that search
/// engines treat unknown URLs as deleted rather than indexing them as real
/// pages ("soft-404" problem). `ResponseOptions` is provided automatically by
/// `leptos_axum` via context when the router is built through
/// [`leptos_axum::LeptosRoutes::leptos_routes`].
///
/// On the client (CSR) there is no HTTP status to set, so the context lookup
/// is gated behind `#[cfg(feature = "ssr")]`.
#[component]
pub fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        use http::StatusCode;

        if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
            response.set_status(StatusCode::NOT_FOUND);
        }
    }

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
