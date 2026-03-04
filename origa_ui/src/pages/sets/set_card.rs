use super::types::SetInfo;
use crate::ui_components::{Button, ButtonVariant, MarkdownText};
use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn SetCard(set_info: SetInfo, on_import: Callback<(String, String)>) -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>().expect("current_user context");

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let description = set_info.description.clone();
    let title = set_info.title.clone();
    view! {
        <div class="set-card">
            <div class="set-card-title">
                {set_info.title.clone()}
            </div>
            <div class="set-card-description">
                <MarkdownText content=Signal::derive(move || description.clone()) known_kanji=known_kanji.get()/>
            </div>
            <div class="set-card-footer">
                <span class="set-card-count">
                    {set_info.word_count.map(|c| format!("{} слов", c)).unwrap_or_default()}
                </span>
                <Button
                    variant=Signal::derive(|| ButtonVariant::Filled)
                    on_click=Callback::new({
                        let set_id = set_info.set_id;
                        let title = title;
                        let on_import = on_import;
                        move |_| on_import.run((set_id.clone(), title.clone()))
                    })
                >
                    "Импорт"
                </Button>
            </div>
        </div>
    }
}
