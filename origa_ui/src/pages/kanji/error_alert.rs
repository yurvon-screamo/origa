use crate::ui_components::{Alert, AlertType};
use leptos::prelude::*;

#[component]
pub fn ErrorAlert(message: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        {move || {
            message.get().map(|msg| view! {
                <Alert alert_type=AlertType::Error message=Signal::derive(move || msg.clone()) />
            })
        }}
    }
}
