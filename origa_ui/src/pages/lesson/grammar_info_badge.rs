use crate::ui_components::{Tag, TagVariant};
use leptos::prelude::*;

#[component]
pub fn GrammarInfoBadge(title: String) -> impl IntoView {
    view! {
        <Tag
            variant=Signal::derive(|| TagVariant::Default)
        >
            <span>
                {title}
            </span>
        </Tag>
    }
}
