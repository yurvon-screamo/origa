use crate::ui_components::{Button, ButtonVariant, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

#[component]
pub fn LevelSelector(
    levels: Vec<JapaneseLevel>,
    selected_level: RwSignal<JapaneseLevel>,
    on_select: Callback<JapaneseLevel>,
) -> impl IntoView {
    view! {
        <div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mb-2".to_string())>
                "Уровень JLPT"
            </Text>
            <div class="flex flex-wrap gap-2">
                <For
                    each=move || levels.clone()
                    key=|level| format!("{:?}", level)
                    children=move |level| {
                        let level_for_btn = level;
                        let is_selected = move || selected_level.get() == level_for_btn;
                        view! {
                            <Button
                                variant=move || if is_selected() { ButtonVariant::Olive } else { ButtonVariant::Default }
                                on_click={let on_select = on_select; Callback::new(move |_| on_select.run(level_for_btn))}
                            >
                                {format!("{:?}", level)}
                            </Button>
                        }
                    }
                />
            </div>
        </div>
    }
}
