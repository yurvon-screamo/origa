use leptos::prelude::*;

#[component]
pub fn Spinner(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] size: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let size_class = Signal::derive(move || {
        match size.get().as_str() {
            "sm" => "spinner-sm",
            "lg" => "spinner-lg",
            _ => "",
        }
        .to_string()
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    view! {
        <div data-testid=test_id_val class=move || format!("spinner {} {}", size_class.get(), class.get())></div>
    }
}

#[component]
pub fn LoadingOverlay(
    #[prop(into)] message: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    view! {
        <div data-testid=test_id_val class=move || format!("loading-overlay anima-page-fade {}", class.get())>
            <Spinner class=Signal::derive(|| "".to_string()) size=Signal::derive(|| "".to_string()) test_id="loading-spinner" />
            <p class="loading-overlay-message">{move || message.get()}</p>
        </div>
    }
}
