use crate::ui_components::{MarkdownText, MarkdownVariant, Text, TextSize};
use leptos::prelude::*;
use origa::domain::QuizOption;
use std::collections::HashSet;

/// The options grid of a single-choice quiz. Rendered on the question
/// side only — after an answer the grid is replaced by the answer
/// ledger strip below the card (direction B, task 3SekkMiYYW), so the
/// per-option result states never render.
#[component]
pub fn QuizOptions(
    options: Vec<QuizOption>,
    selected_option: Option<usize>,
    show_result: Signal<bool>,
    on_select_option: Callback<usize>,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 gap-2 sm:gap-3">
            {move || {
                options
                    .iter()
                    .enumerate()
                    .map(|(index, option): (usize, &QuizOption)| {
                        let base_class = "p-2 sm:p-4 border text-left transition-all cursor-pointer relative flex flex-col justify-center min-h-[4rem] quiz-option-neutral";
                        let selected_ring = if selected_option == Some(index) && !show_result.get() {
                            "ring-2 ring-[var(--accent-olive)]"
                        } else {
                            ""
                        };

                        let class = format!(
                            "{} {} {}",
                            base_class,
                            selected_ring,
                            if show_result.get() { "pointer-events-none" } else { "" }
                        );
                        let key_hint = format!("[{}]", index + 1);
                        let option_text = option.text().to_string();

                        view! {
                            <button
                                class=class
                                data-testid=format!("quiz-option-{}", index)
                                on:click=move |_| {
                                    if !show_result.get() {
                                        on_select_option.run(index);
                                    }
                                }
                            >
                                <Text size=TextSize::Default>
                                    <MarkdownText
                                        content=Signal::derive(move || option_text.clone())
                                        variant=MarkdownVariant::Default
                                        known_kanji=known_kanji.get()
                                        glue_punctuation=true
                                    />
                                </Text>
                                <span class="kbd-hint absolute top-1 right-2 text-[var(--fg-muted)] text-xs font-mono opacity-50 pointer-events-none">
                                    {key_hint}
                                </span>
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}
