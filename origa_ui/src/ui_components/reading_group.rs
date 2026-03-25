use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn ReadingGroup(
    label: &'static str,
    readings: StoredValue<Option<Vec<String>>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let readings_list = move || readings.get_value().unwrap_or_default();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    view! {
        <Show when=move || readings.get_value().is_some()>
            <div
                class="flex gap-4 items-start text-left"
                data-testid=test_id_val
            >
                <div class="w-16 shrink-0 pt-1">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {label}
                    </Text>
                </div>
                <div class="flex gap-2 flex-wrap">
                    <For
                        each=readings_list
                        key=|reading| reading.clone()
                        children=move |reading| {
                            view! {
                                <span class="inline-block px-2 py-1 bg-[var(--bg-aged)] rounded text-sm">
                                    {reading}
                                </span>
                            }
                        }
                    />
                </div>
            </div>
        </Show>
    }
}
