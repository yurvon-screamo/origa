use crate::ui_components::{Checkbox, FuriganaText, Text, TextSize, TypographyVariant};
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

    let status_class = if analyzed_word.is_known {
        "text-sm text-green-600"
    } else {
        "text-sm text-gray-500"
    };

    let status_text = if analyzed_word.is_known {
        "Изв."
    } else {
        "Нов."
    };

    view! {
        <div class="flex justify-between items-center py-1 px-2 rounded bg-[var(--bg-secondary)]">
            <div class="flex items-center gap-2">
                <Checkbox
                    checked=Signal::derive(move || is_selected.get())
                    on_change=Callback::new(move |_| on_toggle.run(()))
                />
                <FuriganaText text=analyzed_word.base_form.clone()/>
                <span class=status_class>{status_text}</span>
            </div>
            {move || {
                analyzed_word.known_meaning.clone().map(|meaning| {
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
