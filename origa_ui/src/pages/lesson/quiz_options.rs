use crate::ui_components::{MarkdownText, MarkdownVariant, Text, TextSize};
use leptos::prelude::*;
use origa::domain::QuizOption;
use std::collections::HashSet;

use super::quiz_result::QuizResult;

#[component]
pub fn QuizOptions(
    options: Vec<QuizOption>,
    selected_option: Option<usize>,
    show_result: bool,
    quiz_result: QuizResult,
    on_select_option: Callback<usize>,
    #[prop(into)] known_kanji: Signal<HashSet<String>>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 gap-2 sm:gap-3">
            {move || {
                options
                    .iter()
                    .enumerate()
                    .map(|(index, option): (usize, &QuizOption)| {
                        let is_correct = option.is_correct();
                        let is_selected = selected_option == Some(index);
                        let base_class = "p-2 sm:p-4 border text-left transition-all cursor-pointer relative flex flex-col justify-center min-h-[4rem]";
                        let disabled_class = if show_result { "pointer-events-none" } else { "" };
                        let result_class = quiz_result.option_class(is_correct, is_selected);
                        let selected_ring = if is_selected && !show_result {
                            "ring-2 ring-[var(--accent-olive)]"
                        } else {
                            ""
                        };

                        let class = format!("{} {} {} {}", base_class, disabled_class, result_class, selected_ring);
                        let key_hint = format!("[{}]", index + 1);
                        let option_text = option.text().to_string();

                        view! {
                            <button
                                class=class
                                on:click=move |_| {
                                    if !show_result {
                                        on_select_option.run(index);
                                    }
                                }
                            >
                                <div class="flex items-start justify-between gap-2">
                                    <Text size=TextSize::Default>
                                        <MarkdownText
                                            content=Signal::derive(move || option_text.clone())
                                            variant=MarkdownVariant::Compact
                                            known_kanji=known_kanji.get()
                                        />
                                    </Text>
                                    <Show when=move || !show_result>
                                        <span class="text-[var(--fg-muted)] text-xs font-mono shrink-0">
                                            {key_hint.clone()}
                                        </span>
                                    </Show>
                                </div>
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}
