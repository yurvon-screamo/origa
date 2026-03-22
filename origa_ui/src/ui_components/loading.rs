use leptos::prelude::*;

#[component]
pub fn Spinner(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] size: Signal<String>,
) -> impl IntoView {
    let size_class = Signal::derive(move || {
        match size.get().as_str() {
            "sm" => "spinner-sm",
            "lg" => "spinner-lg",
            _ => "",
        }
        .to_string()
    });

    view! {
        <div class=move || format!("spinner {} {}", size_class.get(), class.get())></div>
    }
}

#[component]
pub fn LoadingOverlay(
    #[prop(into)] message: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <div class=move || format!("loading-overlay anima-page-fade {}", class.get())>
            <Spinner class=Signal::derive(|| "".to_string()) size=Signal::derive(|| "".to_string()) />
            <p class="loading-overlay-message">{move || message.get()}</p>
        </div>
    }
}
