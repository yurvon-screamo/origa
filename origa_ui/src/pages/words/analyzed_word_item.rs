use crate::ui_components::{
    Card, Checkbox, FuriganaText, Heading, HeadingLevel, MarkdownText, Tag, TagVariant,
};
use leptos::prelude::*;
use origa::application::AnalyzedWord;
use std::collections::HashSet;

#[component]
pub fn AnalyzedWordItem(
    analyzed_word: AnalyzedWord,
    selected_words: RwSignal<HashSet<String>>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    let base_form = analyzed_word.base_form.clone();

    let is_selected = Memo::new(move |_| selected_words.get().contains(&base_form));

    let tag_variant = Signal::derive(move || {
        if analyzed_word.is_known {
            TagVariant::Olive
        } else {
            TagVariant::Default
        }
    });

    let tag_text = Signal::derive(move || {
        if analyzed_word.is_known {
            "Известно".to_string()
        } else {
            "Новое".to_string()
        }
    });

    view! {
        <div class="flex items-start gap-3 p-3 border bg-[var(--bg-paper)]">
            <Checkbox
                checked=Signal::derive(move || is_selected.get())
                on_change=Callback::new(move |_| {
                    on_toggle.run(());
                })
            />
            <Card class=Signal::derive(String::new)>
                <div class="flex flex-col gap-2">
                    <div class="flex items-center gap-2">
                        <Heading level=HeadingLevel::H4>
                            <FuriganaText text=analyzed_word.base_form.clone()/>
                        </Heading>
                        <Tag variant=tag_variant>
                            {tag_text}
                        </Tag>
                    </div>
                    {move || {
                        analyzed_word.known_meaning.clone().map(|meaning| {
                            view! {
                                <MarkdownText content=Signal::derive(move || meaning.clone())/>
                            }
                        })
                    }}
                </div>
            </Card>
        </div>
    }
}
