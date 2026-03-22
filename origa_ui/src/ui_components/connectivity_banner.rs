use crate::store::ConnectivityStore;
use leptos::prelude::*;

#[component]
pub fn ConnectivityBanner() -> impl IntoView {
    let connectivity = use_context::<ConnectivityStore>().expect("ConnectivityStore not found");

    view! {
        <Show when=move || !connectivity.is_online.get()>
            <div
                class="connectivity-banner"
                role="alert"
                aria-live="polite"
            >
                <span class="banner-icon">"⚡"</span>
                <span class="banner-text">"Вы офлайн — изменения сохраняются локально"</span>
            </div>
        </Show>
    }
}
