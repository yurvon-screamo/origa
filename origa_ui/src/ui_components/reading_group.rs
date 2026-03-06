use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn ReadingGroup(
    label: &'static str,
    readings: StoredValue<Option<Vec<String>>>,
) -> impl IntoView {
    let readings_list = move || readings.get_value().unwrap_or_default();

    view! {
        <Show when=move || readings.get_value().is_some()>
            <div class="flex flex-col gap-2 items-center">
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    {label}
                </Text>
                <div class="flex gap-2 flex-wrap justify-center">
                    <For
                        each=readings_list
                        key=|reading| reading.clone()
                        children=move |reading| {
                            view! {
                                <span class="inline-block px-3 py-1.5 bg-[var(--bg-secondary)] rounded-md text-sm">
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
