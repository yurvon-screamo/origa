use crate::i18n::{t, use_i18n};
use crate::store::ConnectivityStore;
use leptos::prelude::*;

#[component]
pub fn ConnectivityBanner(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
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
                <span class="banner-text">{t!(i18n, ui.connectivity_banner)}</span>
            </div>
        </Show>
    }
}
