use crate::i18n::*;
use crate::ui_components::{
    AudioPlayer, Card, MarkdownText, MarkdownVariant, Tag, TagVariant, Text, TextSize,
    TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::QuizOption;
use std::collections::HashSet;

use super::card_type::CardType;
use super::quiz_result::QuizResult;
use super::quiz_result_display::QuizResultDisplay;

#[component]
pub fn PhraseCardView(
    card_type: CardType,
    audio_file: String,
    options: Vec<QuizOption>,
    show_result: bool,
    selected_option: Option<usize>,
    on_select_option: Callback<usize>,
    on_dont_know: Callback<()>,
    dont_know_selected: bool,
    phrase_text: Option<String>,
    #[prop(into)] known_kanji: Signal<HashSet<String>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let audio_src = crate::core::config::cdn_url(&format!("/phrases/audio/{}", audio_file));
    let options_stored = StoredValue::new(options);
    let phrase_text_stored = StoredValue::new(phrase_text);

    let quiz_result = move || {
        if dont_know_selected && show_result {
            return QuizResult::DontKnow;
        }
        if let Some(selected) = selected_option {
            let opts = options_stored.get_value();
            if let Some(opt) = opts.get(selected) {
                return if opt.is_correct() {
                    QuizResult::Correct
                } else {
                    QuizResult::Incorrect
                };
            }
        }
        QuizResult::None
    };

    view! {
        <Card class=Signal::derive(|| "p-4 sm:p-6 min-h-[250px] sm:min-h-[300px] flex flex-col".to_string()) shadow=Signal::derive(|| true)>
            <div class="flex items-center justify-between mb-4">
                <div class="flex items-center gap-2">
                    <Tag variant=Signal::derive(move || card_type.tag_variant())>
                        {card_type.label(&i18n)}
                    </Tag>
                    <Tag variant=Signal::derive(move || TagVariant::Filled)>
                        {t!(i18n, lesson.quiz)}
                    </Tag>
                </div>
            </div>

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-3 sm:mb-6">
                    <AudioPlayer
                        src=audio_src
                        autoplay=true
                        test_id=Signal::derive(|| "phrase-audio-player".to_string())
                    />
                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                        {t!(i18n, lesson.choose_answer)}
                    </Text>
                </div>

                <div class="grid grid-cols-2 gap-2 sm:gap-3">
                    {move || {
                        options_stored
                            .get_value()
                            .iter()
                            .enumerate()
                            .map(|(index, option): (usize, &QuizOption)| {
                                let is_correct = option.is_correct();
                                let is_selected = selected_option == Some(index);
                                let base_class = "p-2 sm:p-4 border text-left transition-all cursor-pointer relative flex flex-col justify-center min-h-[4rem]";
                                let disabled_class = if show_result { "pointer-events-none" } else { "" };
                                let result_class = quiz_result().option_class(is_correct, is_selected);
                                let selected_ring = if is_selected && !show_result {
                                    "ring-2 ring-[var(--accent-olive)]"
                                } else {
                                    ""
                                };
                                let class = format!(
                                    "{} {} {} {}",
                                    base_class,
                                    disabled_class,
                                    result_class,
                                    selected_ring
                                );
                                let key_hint = format!("[{}]", index + 1);
                                let option_text = option.text().to_string();

                                view! {
                                    <button
                                        class=class
                                        data-testid=format!("quiz-option-{}", index)
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
                        <span class="text-[var(--fg-muted)] text-xs font-mono">
                            {t!(i18n, lesson.space_key)}
                        </span>
                    </Show>
                </button>

                <Show when=move || show_result && quiz_result() != QuizResult::DontKnow>
                    <QuizResultDisplay quiz_result=quiz_result() />
                </Show>

                <Show when=move || show_result>
                    {move || match phrase_text_stored.get_value() {
                        Some(text) => view! {
                            <div class="mt-4 p-3 bg-[var(--bg-secondary)] rounded-lg text-center">
                                <Text size=TextSize::Default class="text-[var(--fg-primary)]">
                                    {text}
                                </Text>
                            </div>
                        }.into_any(),
                        None => view! { <div/> }.into_any(),
                    }}
                </Show>
            </div>
        </Card>
    }
}
