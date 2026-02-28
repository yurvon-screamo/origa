use super::types::SetInfo;
use crate::ui_components::{Button, ButtonVariant, MarkdownText};
use leptos::prelude::*;
use origa::domain::WellKnownSets;

#[component]
pub fn SetCard(
    set_info: SetInfo,
    is_importing: bool,
    on_import: Callback<(WellKnownSets, String)>,
) -> impl IntoView {
    let description = set_info.description.clone();
    let title = set_info.title.clone();
    view! {
        <div class="set-card">
            <div class="set-card-title">
                {set_info.title.clone()}
            </div>
            <div class="set-card-description">
                <MarkdownText content=Signal::derive(move || description.clone())/>
            </div>
            <div class="set-card-footer">
                <span class="set-card-count">
                    {format!("{} слов", set_info.word_count)}
                </span>
                <Button
                    variant=Signal::derive(|| ButtonVariant::Filled)
                    on_click=Callback::new({
                        let set = set_info.set;
                        let title = title;
                        let on_import = on_import;
                        move |_| on_import.run((set, title.clone()))
                    })
                    disabled=is_importing
                >
                    {move || if is_importing { "Импорт..." } else { "Импорт" }}
                </Button>
            </div>
        </div>
    }
}
