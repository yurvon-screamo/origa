use leptos::prelude::*;

#[component]
pub fn Spinner(#[prop(optional, into)] class: Signal<String>) -> impl IntoView {
    view! {
        <div class=move || format!("spinner {}", class.get())></div>
    }
}

#[component]
pub fn LoadingOverlay(
    #[prop(into)] message: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <div class=move || format!("loading-overlay {}", class.get())>
            <Spinner class=Signal::derive(|| "".to_string()) />
            <p class="loading-overlay-message">{move || message.get()}</p>
        </div>
    }
}
