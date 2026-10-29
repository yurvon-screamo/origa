use crate::ui_components::{MarkdownText, MarkdownVariant, Text, TextSize};
use leptos::prelude::*;
use origa::domain::QuizOption;
use std::collections::HashSet;

/// The toggle grid of a multi-choice quiz. Rendered on the question
/// side only — after submission the grid is replaced by the answer
/// ledger strip below the card (direction B, task 3SekkMiYYW), so the
/// per-tile result states never render.
#[component]
pub fn QuizOptionsMulti(
    options: Vec<QuizOption>,
    selected_options: HashSet<usize>,
    on_toggle: Callback<usize>,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
) -> impl IntoView {
    let selected_stored = StoredValue::new(selected_options);

    view! {
        <div class="grid grid-cols-2 gap-1 sm:gap-2">
            {options
                .iter()
                .enumerate()
                .map(|(index, option): (usize, &QuizOption)| {
                    let option_text = option.text().to_string();
                    let key_hint = format!("[{}]", index + 1);

                    view! {
                        <MultiOptionButton
                            index=index
                            option_text=option_text
                            key_hint=key_hint
                            selected_options=selected_stored
                            on_toggle=on_toggle
                            known_kanji=known_kanji
                        />
                    }
                })
                .collect::<Vec<_>>()
            }
        </div>
    }
}

#[component]
fn MultiOptionButton(
    index: usize,
    option_text: String,
    key_hint: String,
    selected_options: StoredValue<HashSet<usize>>,
    on_toggle: Callback<usize>,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                let base = "p-1.5 sm:p-2 border text-left transition-all cursor-pointer relative flex flex-col justify-center min-h-[2.25rem]";
                let is_selected = selected_options.get_value().contains(&index);
                if is_selected {
                    format!("{} border-[var(--accent-olive)] bg-[var(--bg-warm)]", base)
                } else {
                    format!("{} border-[var(--border-dark)] bg-[var(--bg-paper)] hover:bg-[var(--bg-aged)]", base)
                }
            }
            data-testid=format!("quiz-option-{}", index)
            on:click=move |_| {
                on_toggle.run(index);
            }
        >
            <div class="flex items-center justify-between w-full">
                <Text size=TextSize::Default>
                    <MarkdownText
                        content=Signal::derive(move || option_text.clone())
                        variant=MarkdownVariant::Default
                        known_kanji=known_kanji.get()
                        glue_punctuation=true
                    />
                </Text>
            </div>
            <Show when=move || selected_options.get_value().contains(&index)>
                <span class="absolute top-1 left-1 w-3 h-3 bg-[var(--accent-olive)]"></span>
            </Show>
            <span class="kbd-hint absolute top-1 right-2 text-[var(--fg-muted)] text-xs font-mono opacity-50 pointer-events-none">
                {key_hint}
            </span>
        </button>
    }
}
