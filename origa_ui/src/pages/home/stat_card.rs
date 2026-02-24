use crate::ui_components::{
    Button, ButtonVariant, Card, DisplayText, Text, TextSize, TypographyVariant,
};
use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
pub fn StatCard(
    #[prop(into)] title: Signal<String>,
    #[prop(into)] value: Signal<String>,
    #[prop(into)] subtitle: Signal<String>,
    #[prop(optional, into)] delta: Signal<String>,
    on_history: Callback<()>,
) -> impl IntoView {
    let has_delta = Signal::derive(move || !delta.get().is_empty());

    view! {
        <Card class=Signal::derive(|| "p-6".to_string())>
            <Text
                size=Signal::derive(|| TextSize::Small)
                variant=Signal::derive(|| TypographyVariant::Muted)
                uppercase=Signal::derive(|| true)
                tracking_widest=Signal::derive(|| true)
                class=Signal::derive(|| "mb-4".to_string())
            >
                {move || title.get()}
            </Text>

            <div class="flex items-baseline gap-2 mb-2">
                <DisplayText class=Signal::derive(|| "".to_string())>
                    {move || value.get()}
                </DisplayText>
                <Show when=move || has_delta.get()>
                    <span class="text-sm font-mono text-green-600">
                        {move || delta.get()}
                    </span>
                </Show>
            </div>

            <Text
                size=Signal::derive(|| TextSize::Small)
                variant=Signal::derive(|| TypographyVariant::Muted)
                class=Signal::derive(|| "mb-4".to_string())
            >
                {move || subtitle.get()}
            </Text>

            <Button
                variant=Signal::derive(|| ButtonVariant::Ghost)
                on_click=Callback::new(move |_: MouseEvent| on_history.run(()))
            >
                "История"
            </Button>
        </Card>
    }
}
