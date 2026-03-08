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
    let title_for_display = set_info.title.clone();
    let is_imported = set_info.is_imported;
    let word_count = set_info.word_count;
    let set_id_for_callback = set_info.set_id.clone();
    let title_for_callback = set_info.title.clone();

    view! {
        <div class="set-card">
            <div class="set-card-title">
                {title_for_display.clone()}
                <Show when=move || is_imported>
                    <span class="set-imported ml-2">
                        "Импортирован"
                    </span>
                </Show>
            </div>
            <div class="set-card-description">
                <MarkdownText content=Signal::derive(move || description.clone()) known_kanji=known_kanji.get()/>
            </div>
            <div class="set-card-footer">
                <span class="set-card-count">
                    {word_count.map(|c| format!("{} слов", c)).unwrap_or_default()}
                </span>
                <Show when=move || !is_imported>
                    <SetCardButton
                        set_id=set_id_for_callback.clone()
                        title=title_for_callback.clone()
                        on_import=on_import
                    />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn SetCardButton(
    set_id: String,
    title: String,
    on_import: Callback<(String, String)>,
) -> impl IntoView {
    view! {
        <Button
            variant=Signal::derive(|| ButtonVariant::Filled)
            on_click=Callback::new({
                let set_id = set_id;
                let title = title;
                move |_| on_import.run((set_id.clone(), title.clone()))
            })
        >
            "Импорт"
        </Button>
    }
}
