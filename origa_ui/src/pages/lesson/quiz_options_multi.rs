use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, MarkdownText, MarkdownVariant, Text, TextSize};
use leptos::prelude::*;
use origa::domain::{MultiQuizResult, QuizOption};
use std::collections::HashSet;

use super::quiz_result::OptionDisplay;

#[component]
pub fn QuizOptionsMulti(
    options: Vec<QuizOption>,
    selected_options: HashSet<usize>,
    show_result: bool,
    multi_submitted: bool,
    multi_result: Option<MultiQuizResult>,
    on_toggle: Callback<usize>,
    on_submit: Callback<()>,
    on_dont_know: Callback<()>,
    dont_know_selected: bool,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    #[prop(default = false)] waiting_for_next: bool,
    #[prop(default = Callback::new(|_: ()| {}))] on_next_card: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let selected_options_stored = StoredValue::new(selected_options);
    let multi_result_stored = StoredValue::new(multi_result);
    let tags: Vec<Option<String>> = options
        .iter()
        .map(|o| o.tag().map(|t| t.to_string()))
        .collect();

    view! {
        <div class="grid grid-cols-2 gap-2 sm:gap-3">
            {options
                .iter()
                .enumerate()
                .map(|(index, option): (usize, &QuizOption)| {
                    let option_text = option.text().to_string();
                    let key_hint = format!("{}", index + 1);
                    let is_correct = option.is_correct();
                    let tag = tags.get(index).cloned().flatten();

                    view! {
                        <MultiOptionButton
                            index=index
                            option_text=option_text
                            key_hint=key_hint
                            is_correct=is_correct
                            tag=tag
                            selected_options=selected_options_stored
                            multi_submitted=multi_submitted
                            multi_result_stored=multi_result_stored
                            on_toggle=on_toggle
                            show_result=show_result
                            known_kanji=known_kanji
                        />
                    }
                })
                .collect::<Vec<_>>()
            }
        </div>

        <Show when=move || !multi_submitted && !dont_know_selected>
            <button
                data-testid="quiz-submit-btn"
                class=move || {
                    let base = "w-full mt-2 p-2 sm:p-4 border text-center transition-all flex items-center justify-center gap-2 font-serif font-medium";
                    if selected_options_stored.get_value().is_empty() {
                        format!("{} opacity-40 cursor-not-allowed border-[var(--border-dark)] bg-[var(--bg-paper)]", base)
                    } else {
                        format!("{} cursor-pointer bg-[var(--fg-black)] border-[var(--fg-black)] hover:opacity-80", base)
                    }
                }
                style=move || {
                    if selected_options_stored.get_value().is_empty() {
                        "color: var(--fg-black)".to_string()
                    } else {
                        "color: var(--bg-paper)".to_string()
                    }
                }
                disabled=move || selected_options_stored.get_value().is_empty()
                on:click=move |_| {
                    on_submit.run(());
                }
            >
                <span>{t!(i18n, lesson.check)}</span>
                <span class="text-xs font-mono opacity-50">{t!(i18n, lesson.enter_key)}</span>
            </button>
        </Show>

        <Show when=move || !multi_submitted>
            <button
                data-testid="quiz-dont-know-btn"
                class=move || {
                    let base = "w-full mt-2 p-2 sm:p-4 border text-center transition-all cursor-pointer flex items-center justify-center gap-2";
                    if dont_know_selected {
                        format!("{} quiz-option-neutral ring-2 ring-[var(--accent-olive)]", base)
                    } else if show_result {
                        format!("{} quiz-option-dimmed pointer-events-none", base)
                    } else {
                        format!("{} quiz-option-neutral", base)
                    }
                }
                on:click=move |_| {
                    if !show_result {
                        on_dont_know.run(());
                    }
                }
            >
                <Text size=TextSize::Default>{t!(i18n, lesson.dont_know)}</Text>
                <Show when=move || !show_result>
                    <span class="text-[var(--fg-muted)] text-xs font-mono">{t!(i18n, lesson.space_key)}</span>
                </Show>
            </button>
        </Show>

        <Show when=move || waiting_for_next && multi_submitted>
            <div class="mt-4 flex justify-center">
                <Button
                    variant=Signal::derive(|| ButtonVariant::Filled)
                    on_click=Callback::new(move |_| on_next_card.run(()))
                    test_id=Signal::derive(|| "quiz-next-btn".to_string())
                >
                    <span>{t!(i18n, lesson.next)}</span>
                    <span class="text-[var(--fg-light)]">{t!(i18n, lesson.space_key)}</span>
                </Button>
            </div>
        </Show>
    }
}

#[component]
fn MultiOptionButton(
    index: usize,
    option_text: String,
    key_hint: String,
    is_correct: bool,
    tag: Option<String>,
    selected_options: StoredValue<HashSet<usize>>,
    multi_submitted: bool,
    multi_result_stored: StoredValue<Option<MultiQuizResult>>,
    on_toggle: Callback<usize>,
    show_result: bool,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
) -> impl IntoView {
    let tag_stored = StoredValue::new(tag);

    view! {
        <button
            class=move || {
                if multi_submitted {
                    if let Some(result) = multi_result_stored.get_value() {
                        let is_selected = selected_options.get_value().contains(&index);
                        let display = super::quiz_result::QuizResult::multi_option_display(
                            is_correct,
                            is_selected,
                            &result,
                            index,
                        );
                        return display.to_class();
                    }
                }
                let base = "p-2 sm:p-4 border text-left transition-all cursor-pointer relative flex flex-col justify-center min-h-[4rem]";
                let is_selected = selected_options.get_value().contains(&index);
                if is_selected {
                    format!("{} border-[var(--accent-olive)] bg-[var(--bg-warm)]", base)
                } else {
                    format!("{} border-[var(--border-dark)] bg-[var(--bg-paper)] hover:bg-[var(--bg-aged)]", base)
                }
            }
            data-testid=format!("quiz-option-{}", index)
            on:click=move |_| {
                if !show_result {
                    on_toggle.run(index);
                }
            }
        >

            <div class="flex items-center justify-between w-full">
                <Text size=TextSize::Default>
                    <MarkdownText
                        content=Signal::derive(move || option_text.clone())
                        variant=MarkdownVariant::Default
                        known_kanji=known_kanji.get()
                    />
                </Text>
                <Show when=move || multi_submitted && tag_stored.get_value().is_some()>
                    {move || {
                        tag_stored.get_value().map(|t| {
                    let tag_class = tag_class(t.as_str());
                            view! {
                                <span class=tag_class>{t}</span>
                            }
                        })
                    }}
                </Show>
            </div>
            <Show when=move || !multi_submitted>
                <span class="absolute top-1 right-2 text-[var(--fg-muted)] text-xs font-mono opacity-50 pointer-events-none">
                    {key_hint.clone()}
                </span>
            </Show>
            <Show when=move || selected_options.get_value().contains(&index) && !multi_submitted>
                <span class="absolute top-1 left-1 w-3 h-3 bg-[var(--accent-olive)]"></span>
            </Show>
        </button>
    }
}

fn tag_class(tag: &str) -> &'static str {
    match tag {
        "ON" => "tag tag-olive ml-2",
        "KUN" => "tag tag-sage ml-2",
        _ => "tag ml-2",
    }
}

impl OptionDisplay {
    fn to_class(self) -> String {
        let base = "p-2 sm:p-4 border text-left relative flex flex-col justify-center min-h-[4rem]";
        match self {
            OptionDisplay::Correct => {
                format!(
                    "{} border-[var(--success)] border-l-4 border-l-[var(--success)] bg-[var(--bg-warm)] pointer-events-none",
                    base
                )
            },
            OptionDisplay::Wrong => {
                format!(
                    "{} border-[var(--error)] border-l-4 border-l-[var(--error)] bg-[var(--bg-warm)] pointer-events-none anima-shake",
                    base
                )
            },
            OptionDisplay::Missed => {
                format!("{} quiz-option-missed pointer-events-none", base)
            },
            OptionDisplay::Dimmed => {
                format!("{} opacity-50 pointer-events-none", base)
            },
            OptionDisplay::Neutral => base.to_string(),
        }
    }
}
