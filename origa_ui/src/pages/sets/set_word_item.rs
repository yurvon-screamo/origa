use crate::ui_components::{Checkbox, FuriganaText, Text, TextSize, TypographyVariant};
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

    let status_class = if is_known {
        "text-sm text-green-600"
    } else {
        "text-sm text-gray-500"
    };

    let status_text = if is_known { "Изв." } else { "Нов." };

    view! {
        <div class="flex justify-between items-center py-1 px-2 rounded bg-[var(--bg-secondary)]">
            <div class="flex items-center gap-2">
                <Checkbox
                    checked=Signal::derive(move || is_selected.get())
                    on_change=Callback::new(move |_| on_toggle.run(()))
                />
                <FuriganaText text=word.clone()/>
                <span class=status_class>{status_text}</span>
            </div>
            {move || {
                known_meaning.clone().map(|meaning| {
                    view! {
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {meaning}
                        </Text>
                    }
                })
            }}
        </div>
    }
}
