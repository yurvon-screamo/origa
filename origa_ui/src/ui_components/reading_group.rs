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
                class="reading-group"
                data-testid=test_id_val
            >
                <div class="reading-kanji">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {label}
                    </Text>
                </div>
                <div class="reading-furigana">
                    <For
                        each=readings_list
                        key=|reading| reading.clone()
                        children=move |reading| {
                            view! {
                                <span class="reading-tag">
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
