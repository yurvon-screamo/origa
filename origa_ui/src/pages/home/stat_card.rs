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
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_history = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "stat-history".to_string()
        } else {
            format!("{}-history", val)
        }
    });

    let has_delta = Signal::derive(move || !delta.get().is_empty());

    let delta_class = Signal::derive(move || {
        let d = delta.get();
        if d.is_empty() {
            String::new()
        } else if d.starts_with('-') {
            "text-sm font-mono text-[var(--error)]".to_string()
        } else {
            "text-sm font-mono text-[var(--success)]".to_string()
        }
    });

    let test_id_for_card = Signal::derive(move || test_id.get());

    view! {
        <Card class=Signal::derive(|| "p-6".to_string()) test_id=test_id_for_card>
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
                    <span class=move || delta_class.get()>
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
                test_id=test_id_history
            >
                "История"
            </Button>
        </Card>
    }
}
