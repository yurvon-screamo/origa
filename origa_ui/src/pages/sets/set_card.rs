use super::types::SetInfo;
use crate::ui_components::{
    Button, ButtonSize, ButtonVariant, Card, Checkbox, Heading, HeadingLevel, MarkdownText, Tag,
    TagVariant, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub fn SetCard(
    set_info: SetInfo,
    known_kanji: HashSet<String>,
    on_import: Callback<(String, String)>,
    selected_sets: RwSignal<HashSet<String>>,
    on_toggle_select: Callback<String>,
) -> impl IntoView {
    let description = set_info.description.clone();
    let title_for_display = set_info.title.clone();
    let is_imported = set_info.is_imported;
    let word_count = set_info.word_count;
    let set_id = set_info.set_id.clone();

    let is_selected = Signal::derive({
        let sid = set_id.clone();
        let ss = selected_sets;
        move || ss.get().contains(&sid)
    });

    let card_class = Signal::derive(move || {
        let base = "p-4 flex flex-col h-full transition-all duration-200".to_string();
        if is_selected.get() {
            format!("{} ring-2 ring-primary ring-offset-2", base)
        } else {
            base
        }
    });

    let toggle_callback = Callback::new({
        let sid = set_id.clone();
        move |_| {
            on_toggle_select.run(sid.clone());
        }
    });

    view! {
        <Card class=card_class test_id="sets-card-item">
            <div class="flex items-center justify-between gap-2 mb-2">
                <div class="flex items-center gap-2 flex-1 min-w-0">
                    <Show when=move || !is_imported>
                        <Checkbox
                            checked=is_selected
                            on_change=toggle_callback
                        />
                    </Show>
                    <Heading level=Signal::derive(|| HeadingLevel::H4) class="truncate">
                        {title_for_display}
                    </Heading>
                </div>
                <Show when=move || !is_imported>
                    <SetCardButton
                        set_id=set_info.set_id.clone()
                        title=set_info.title.clone()
                        on_import=on_import
                    />
                </Show>
            </div>
            <div class="flex-1 min-h-0 mb-3">
                <MarkdownText content=Signal::derive(move || description.clone()) known_kanji=known_kanji.clone()/>
            </div>
            <div class="flex items-center justify-between mt-auto">
                <Text size=Signal::derive(|| TextSize::Small) variant=Signal::derive(|| TypographyVariant::Muted)>
                    {word_count.map(|c| format!("{} слов", c)).unwrap_or_default()}
                </Text>
                <Show when=move || is_imported>
                    <Tag variant=Signal::derive(|| TagVariant::Olive)>
                        "Импортирован"
                    </Tag>
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
            size=ButtonSize::Small
            on_click=Callback::new(move |_| on_import.run((set_id.clone(), title.clone())))
            test_id="sets-card-import-btn"
        >
            "Импорт"
        </Button>
    }
}
