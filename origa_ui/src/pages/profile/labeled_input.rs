use crate::ui_components::{Input, Text, TextSize};
use leptos::prelude::*;

#[component]
pub fn LabeledInput(
    label: String,
    value: RwSignal<String>,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    view! {
        <div>
            <Text size={TextSize::Large}>
                {label}
            </Text>
            <Input
                value={value}
                disabled={disabled}
            />
        </div>
    }
}
