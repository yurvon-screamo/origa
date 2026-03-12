use super::types::SetInfo;
use crate::ui_components::{
    Button, ButtonVariant, Card, Heading, HeadingLevel, MarkdownText, Tag, TagVariant, Text,
    TextSize, TypographyVariant,
};
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

    view! {
        <Card class=Signal::derive(|| "p-4 flex flex-col h-full".to_string())>
            <div class="flex items-center gap-2 mb-2">
                <Heading level=Signal::derive(|| HeadingLevel::H4)>
                    {title_for_display}
                </Heading>
                <Show when=move || is_imported>
                    <Tag variant=Signal::derive(|| TagVariant::Olive)>
                        "Импортирован"
                    </Tag>
                </Show>
            </div>
            <div class="flex-1 min-h-0 mb-3">
                <MarkdownText content=Signal::derive(move || description.clone()) known_kanji=known_kanji.get()/>
            </div>
            <div class="flex justify-between items-center mt-auto pt-2 border-t border-[var(--border-color)]">
                <Text size=Signal::derive(|| TextSize::Small) variant=Signal::derive(|| TypographyVariant::Muted)>
                    {word_count.map(|c| format!("{} слов", c)).unwrap_or_default()}
                </Text>
                <Show when=move || !is_imported>
                    <SetCardButton
                        set_id=set_info.set_id.clone()
                        title=set_info.title.clone()
                        on_import=on_import
                    />
                </Show>
            </div>
        </Card>
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
            on_click=Callback::new(move |_| on_import.run((set_id.clone(), title.clone())))
        >
            "Импорт"
        </Button>
    }
}
