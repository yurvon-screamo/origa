use crate::store::ConnectivityStore;
use leptos::prelude::*;

#[component]
pub fn ConnectivityBanner(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let connectivity = use_context::<ConnectivityStore>().expect("ConnectivityStore not found");

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <Show when=move || !connectivity.is_online.get()>
            <div
                class="connectivity-banner"
                role="alert"
                aria-live="polite"
                data-testid=test_id_val
            >
                <span class="banner-icon">"⚡"</span>
                <span class="banner-text">"Вы офлайн — изменения сохраняются локально"</span>
            </div>
        </Show>
    }
}
