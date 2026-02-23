use crate::ui_components::{Alert, AlertType};
use leptos::prelude::*;

#[component]
pub fn ErrorMessage(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <Alert
            alert_type=AlertType::Error
            title="Ошибка"
            message=message
        />
    }
}
