use crate::ui_components::{Alert, AlertType};
use leptos::prelude::*;

#[component]
pub fn ErrorAlert(message: RwSignal<Option<String>>) -> impl IntoView {
    move || {
        message.get().map(|msg| {
            view! {
                <Alert
                    alert_type=Signal::from(AlertType::Error)
                    title=Signal::derive(|| "Ошибка".to_string())
                    message=Signal::derive(move || msg.clone())
                />
            }
        })
    }
}
