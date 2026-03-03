use crate::ui_components::{
    get_reading_from_text, is_speech_supported, speak_text, AudioButtons, Card, DisplayText,
    FuriganaText, Heading, HeadingLevel, KanjiViewMode, KanjiWritingSection, MarkdownText,
    MarkdownVariant, Tag, TagVariant, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, QuizCard, QuizOption};

use super::lesson_card::CardType;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum QuizResult {
    #[default]
    None,
    Correct,
    Incorrect,
}

impl QuizResult {
    pub fn option_class(&self, is_correct: bool) -> &'static str {
        match (self, is_correct) {
            (QuizResult::None, _) => {
                "bg-[var(--bg-paper)] hover:bg-[var(--bg-aged)] border-[var(--border-dark)]"
            }
            (QuizResult::Correct, true) | (QuizResult::Incorrect, true) => {
                "bg-[var(--bg-warm)] border-[var(--success)] text-[var(--success)]"
            }
            (QuizResult::Correct, false) => {
                "bg-[var(--bg-paper)] border-[var(--border-light)] opacity-50"
            }
            (QuizResult::Incorrect, false) => {
                "bg-[var(--bg-warm)] border-[var(--error)] text-[var(--error)]"
            }
        }
    }
}

#[component]
pub fn QuizCardView(
    quiz_card: QuizCard,
    show_result: bool,
    selected_option: Option<usize>,
    on_select_option: Callback<usize>,
) -> impl IntoView {
    let card = quiz_card.card().clone();
    let card_type = CardType::from(&card);
    let question = StoredValue::new(card.question().text().to_string());
    let options: StoredValue<Vec<QuizOption>> = StoredValue::new(quiz_card.options().to_vec());

    let quiz_result = move || {
        if let Some(selected) = selected_option {
            let opts = options.get_value();
            if let Some(opt) = opts.get(selected) {
                if opt.is_correct() {
                    return QuizResult::Correct;
                } else {
                    return QuizResult::Incorrect;
                }
            }
        }
        QuizResult::None
    };

    let kanji_for_animation: StoredValue<Option<String>> = StoredValue::new(match &card {
        DomainCard::Kanji(_) => Some(card.question().text().to_string()),
        _ => None,
    });

    let lesson_ctx = use_context::<crate::pages::lesson::lesson_state::LessonContext>();
    let question_text = question.get_value();

    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get())
            .unwrap_or(false);
        if !show_result && card_type != CardType::Kanji && is_speech_supported() && !is_muted {
            let reading = get_reading_from_text(&question_text);
            let _ = speak_text(&reading, 1.0);
        }
    });

    view! {
        <Card class=Signal::derive(|| "p-6 min-h-[300px] flex flex-col".to_string()) shadow=Signal::derive(|| true)>
            <div class="flex items-center justify-between mb-4">
                <div class="flex items-center gap-2">
                    <Tag variant=Signal::derive(move || card_type.tag_variant())>
                        {card_type.label()}
                    </Tag>
                    <Tag variant=Signal::derive(move || TagVariant::Filled)>
                        "Тест"
                    </Tag>
                </div>
                <Show when=move || card_type != CardType::Kanji>
                    <AudioButtons
                        text=question.get_value()
                        class=Signal::derive(|| "".to_string())
                    />
                </Show>
            </div>

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-6">
                    <Show when=move || card_type != CardType::Kanji>
                        <div class="mb-4">
                            <Heading level=HeadingLevel::H2>
                                <FuriganaText text=question.get_value()/>
                            </Heading>
                        </div>
                    </Show>

                    <Show when=move || kanji_for_animation.get_value().is_some()>
                        {move || {
                            kanji_for_animation.get_value().map(|kanji: String| {
                                let kanji_for_section = kanji.clone();
                                view! {
                                    <div class="mb-6">
                                        <DisplayText>
                                            {kanji}
                                        </DisplayText>
                                    </div>
                                    <KanjiWritingSection kanji=kanji_for_section mode=KanjiViewMode::Animation />
                                }
                            })
                        }}
                    </Show>

                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                        "Выберите правильный ответ:"
                    </Text>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    {move || {
                        let opts = options.get_value();
                        let current_result = quiz_result();
                        opts.into_iter()
                            .enumerate()
                            .map(|(index, option): (usize, QuizOption)| {
                                let is_correct = option.is_correct();
                                let is_selected = selected_option == Some(index);
                                let base_class = "p-4 rounded-lg border-2 text-left transition-all cursor-pointer relative";
                                let disabled_class = if show_result { "pointer-events-none" } else { "" };
                                let result_class = current_result.option_class(is_correct);
                                let selected_ring = if is_selected && !show_result { "ring-2 ring-[var(--accent-olive)]" } else { "" };

                                let class = format!("{} {} {} {}", base_class, disabled_class, result_class, selected_ring);
                                let key_hint = format!("[{}]", index + 1);

                                let key_hint_clone = key_hint.clone();
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
                                                />
                                            </Text>
                                            <Show when=move || !show_result>
                                                <span class="text-[var(--fg-muted)] text-xs font-mono shrink-0">
                                                    {key_hint_clone.clone()}
                                                </span>
                                            </Show>
                                        </div>
                                    </button>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>

                <Show when=move || show_result>
                    <div class="mt-6 text-center">
                        <Text size=TextSize::Default class=move || {
                            match quiz_result() {
                                QuizResult::Correct => "text-[var(--success)] font-bold".to_string(),
                                QuizResult::Incorrect => "text-[var(--error)] font-bold".to_string(),
                                QuizResult::None => "".to_string(),
                            }
                        }>
                            {move || match quiz_result() {
                                QuizResult::Correct => "✓ Правильно!",
                                QuizResult::Incorrect => "✗ Неверно",
                                QuizResult::None => "",
                            }}
                        </Text>
                    </div>
                </Show>
            </div>
        </Card>
    }
}
