use super::types::SetInfo;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use origa::domain::WellKnownSets;

#[component]
pub fn SetCard(
    set_info: SetInfo,
    is_importing: bool,
    on_import: Callback<WellKnownSets>,
) -> impl IntoView {
    view! {
        <div class="set-card">
            <div class="set-card-title">
                {set_info.title.clone()}
            </div>
            <div class="set-card-description">
                {set_info.description.clone()}
            </div>
            <div class="set-card-footer">
                <span class="set-card-count">
                    {format!("{} слов", set_info.word_count)}
                </span>
                <Button
                    variant=Signal::derive(|| ButtonVariant::Filled)
                    on_click=Callback::new({
                        let set = set_info.set;
                        let on_import = on_import;
                        move |_| on_import.run(set)
                    })
                    disabled=is_importing
                >
                    {move || if is_importing { "Импорт..." } else { "Импорт" }}
                </Button>
            </div>
        </div>
    }
}
