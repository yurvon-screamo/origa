use crate::ui_components::{Tag, TagVariant};
use leptos::prelude::*;

#[component]
pub fn GrammarInfoBadge(title: String, description: String) -> impl IntoView {
    view! {
        <Tag
            variant=Signal::derive(|| TagVariant::Default)
            class=Signal::derive(|| "cursor-help".to_string())
        >
            <span title=description>
                {format!("Грамматика: {}", title)}
            </span>
        </Tag>
    }
}
