use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

#[component]
pub fn LevelSelector(selected_level: RwSignal<JapaneseLevel>) -> impl IntoView {
    let levels = [
        JapaneseLevel::N5,
        JapaneseLevel::N4,
        JapaneseLevel::N3,
        JapaneseLevel::N2,
        JapaneseLevel::N1,
    ];

    view! {
        <div class="flex space-x-2 mt-2">
            <For
                each=move || levels.to_vec()
                key=|level| format!("{:?}", level)
                children=move |level| {
                    let level_for_selector = level.clone();
                    let is_selected = move || selected_level.get() == level_for_selector;
                    view! {
                        <Button
                            variant=move || if is_selected() { ButtonVariant::Olive } else { ButtonVariant::Default }
                            on_click={Callback::new(move |_| selected_level.set(level_for_selector.clone()))}
                        >
                            {format!("{:?}", level)}
                        </Button>
                    }
                }
            />
        </div>
    }
}
