use crate::i18n::use_i18n;
use crate::ui_components::{Alert, AlertType};
use leptos::prelude::*;

#[component]
pub fn ErrorAlert(message: RwSignal<Option<String>>) -> impl IntoView {
    let i18n = use_i18n();
    move || {
        message.get().map(move |msg| {
            view! {
                <Alert
                    alert_type=Signal::from(AlertType::Error)
                    title=Signal::derive(move || i18n.get_keys().common().error().inner().to_string())
                    message=Signal::derive(move || msg.clone())
                />
            }
        })
    }
}
