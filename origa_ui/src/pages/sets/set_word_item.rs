use crate::ui_components::{
    Card, Checkbox, FuriganaText, Heading, HeadingLevel, Tag, TagVariant, Text, TextSize,
    TypographyVariant,
};
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub fn SetWordItem(
    word: String,
    known_meaning: Option<String>,
    is_known: bool,
    selected_words: RwSignal<HashSet<String>>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    let word_for_memo = word.clone();
    let is_selected = Memo::new(move |_| selected_words.get().contains(&word_for_memo));

    let tag_variant = Signal::derive(move || {
        if is_known {
            TagVariant::Olive
        } else {
            TagVariant::Default
        }
    });

    let tag_text = Signal::derive(move || {
        if is_known {
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
                            <FuriganaText text=word.clone()/>
                        </Heading>
                        <Tag variant=tag_variant>
                            {tag_text}
                        </Tag>
                    </div>
                    {move || {
                        known_meaning.clone().map(|meaning| {
                            view! {
                                <Text size=TextSize::Large variant=TypographyVariant::Primary>
                                    {meaning}
                                </Text>
                            }
                        })
                    }}
                </div>
            </Card>
        </div>
    }
}
