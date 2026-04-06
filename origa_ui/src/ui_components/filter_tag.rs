use crate::ui_components::{Tag, TagVariant};
use leptos::prelude::*;

#[component]
pub fn FilterTag(
    label: String,
    is_active: Signal<bool>,
    on_click: Callback<leptos::ev::MouseEvent>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    view! {
        <Tag
            variant=Signal::derive(move || {
                if is_active.get() {
                    TagVariant::Filled
                } else {
                    TagVariant::Default
                }
            })
            class=Signal::derive(|| "cursor-pointer".to_string())
            test_id=test_id
            on_click=on_click
        >
            {label}
        </Tag>
    }
}
