use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn ErrorAlert(message: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        <Show when=move || message.get().is_some()>
            <div class="p-3 bg-red-100 border border-red-300 rounded-lg">
                <Text size=TextSize::Small variant=TypographyVariant::Primary class="text-red-700">
                    {move || message.get().unwrap_or_default()}
                </Text>
            </div>
        </Show>
    }
}
