use crate::ui_components::{Input, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn EmailInput(value: RwSignal<String>, on_enter: Callback<()>) -> impl IntoView {
    view! {
        <div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true class="block mb-2">
                "Email"
            </Text>
            <Input
                value=value
                placeholder="email@example.com"
                on_change=Callback::new(move |ev: leptos::ev::Event| {
                    value.set(event_target_value(&ev));
                })
                on_keydown=Callback::new(move |ev: leptos::ev::KeyboardEvent| {
                    if ev.key() == "Enter" {
                        on_enter.run(());
                    }
                })
            />
        </div>
    }
}
