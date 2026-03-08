use crate::ui_components::{Alert, AlertType};
use leptos::prelude::*;

#[component]
pub fn GrammarInfoBadge(title: String, description: String) -> impl IntoView {
    let title_signal = Signal::derive(move || format!("С грамматикой {}", title.clone()));
    let description_signal = Signal::derive(move || description.clone());

    view! {
        <div class="mb-4">
            <Alert
                alert_type=Signal::derive(|| AlertType::Info)
                title=title_signal
                message=description_signal
            />
        </div>
    }
}
