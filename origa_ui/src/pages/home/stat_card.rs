use crate::ui_components::{Card, DisplayText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn QuickStatCard(
    #[prop(into)] title: Signal<String>,
    #[prop(into)] value: Signal<String>,
    #[prop(optional, into)] delta: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    on_card_click: Callback<()>,
) -> impl IntoView {
    let has_delta = Signal::derive(move || !delta.get().is_empty());

    let delta_class = Signal::derive(move || {
        let d = delta.get();
        if d.is_empty() {
            String::new()
        } else if d.starts_with('-') {
            "text-[var(--error)]".to_string()
        } else {
            "text-[var(--success)]".to_string()
        }
    });

    view! {
        <div
            class="cursor-pointer"
            on:click=move |_: leptos::ev::MouseEvent| on_card_click.run(())
        >
            <Card
                class=Signal::derive(|| "interactive p-4".to_string())
                test_id=test_id
            >
                <Text
                    size=Signal::from(TextSize::Small)
                    variant=Signal::from(TypographyVariant::Muted)
                    uppercase=Signal::from(true)
                    tracking_widest=Signal::from(true)
                    class=Signal::derive(|| "mb-4".to_string())
                >
                    {move || title.get()}
                </Text>

                <div class="flex items-baseline gap-2">
                    <DisplayText class=Signal::derive(String::new)>
                        {move || value.get()}
                    </DisplayText>
                    <Show when=move || has_delta.get()>
                        <Text
                            size=Signal::from(TextSize::Small)
                            class=delta_class
                            test_id=Signal::derive(|| "delta-badge".to_string())
                        >
                            {move || delta.get()}
                        </Text>
                    </Show>
                </div>
            </Card>
        </div>
    }
}
