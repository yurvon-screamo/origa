use crate::ui_components::{
    Card, DisplayText, FuriganaText, Heading, HeadingLevel, Text, TextSize, TypographyVariant,
    get_reading_from_text, is_speech_supported, speak_text,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, YesNoCard};
use std::collections::HashSet;

use super::card_type::CardType;
use super::quiz_card_header::QuizCardHeader;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum YesNoResult {
    #[default]
    None,
    Correct,
    Incorrect,
}

impl YesNoResult {
    pub fn from_answer(is_correct: bool, user_said_yes: bool, showing_result: bool) -> Self {
        if !showing_result {
            return YesNoResult::None;
        }
        let answered_correctly = (is_correct && user_said_yes) || (!is_correct && !user_said_yes);
        if answered_correctly {
            YesNoResult::Correct
        } else {
            YesNoResult::Incorrect
        }
    }

    pub fn result_text(&self) -> &'static str {
        match self {
            YesNoResult::Correct => "✓ Правильно!",
            YesNoResult::Incorrect => "✗ Неверно",
            YesNoResult::None => "",
        }
    }

    pub fn result_class(&self) -> &'static str {
        match self {
            YesNoResult::Correct => "text-[var(--success)] font-bold",
            YesNoResult::Incorrect => "text-[var(--error)] font-bold",
            YesNoResult::None => "",
        }
    }
}

#[component]
pub fn YesNoCardView(
    yesno_card: YesNoCard,
    show_result: bool,
    selected_answer: Option<bool>,
    on_answer: Callback<bool>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<String>>,
) -> impl IntoView {
    let card = yesno_card.card().clone();
    let card_type = CardType::from(&card);
    let lang = native_language;
    let statement = StoredValue::new(yesno_card.statement_text().to_string());
    let is_statement_correct = yesno_card.is_correct();

    let question_text = StoredValue::new(
        card.question(&lang)
            .ok()
            .map(|q| q.text().to_string())
            .unwrap_or_default(),
    );

    let kanji_for_animation: StoredValue<Option<String>> = StoredValue::new(match &card {
        DomainCard::Kanji(_) => Some(
            card.question(&lang)
                .ok()
                .map(|q| q.text().to_string())
                .unwrap_or_default(),
        ),
        _ => None,
    });

    let lesson_ctx = use_context::<super::lesson_state::LessonContext>();
    let stmt_for_effect = statement.get_value();
    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get())
            .unwrap_or(false);
        if !show_result && card_type != CardType::Kanji && is_speech_supported() && !is_muted {
            let reading = get_reading_from_text(&stmt_for_effect);
            let _ = speak_text(&reading, 1.0);
        }
    });

    let yesno_result = move || {
        YesNoResult::from_answer(
            is_statement_correct,
            selected_answer.unwrap_or(false),
            show_result,
        )
    };

    let yes_selected = selected_answer == Some(true);
    let no_selected = selected_answer == Some(false);

    let yes_class = move || {
        let base =
            "p-3 sm:p-4 border text-lg transition-all cursor-pointer flex-1 text-center rounded-lg"
                .to_string();
        let mut classes = base;

        if show_result {
            classes.push_str(" pointer-events-none");
        }

        if yes_selected {
            classes.push_str(" ring-2 ring-[var(--accent-olive)]");
        }

        if show_result {
            let correct_answer = is_statement_correct;
            if correct_answer {
                classes.push_str(" bg-[var(--success)] bg-opacity-20 border-[var(--success)]");
            } else if yes_selected {
                classes.push_str(" bg-[var(--error)] bg-opacity-20 border-[var(--error)]");
            } else {
                classes.push_str(" opacity-50");
            }
        } else if yes_selected {
            classes
                .push_str(" bg-[var(--accent-olive)] bg-opacity-10 border-[var(--accent-olive)]");
        }

        classes
    };

    let no_class = move || {
        let base =
            "p-3 sm:p-4 border text-lg transition-all cursor-pointer flex-1 text-center rounded-lg"
                .to_string();
        let mut classes = base;

        if show_result {
            classes.push_str(" pointer-events-none");
        }

        if no_selected {
            classes.push_str(" ring-2 ring-[var(--accent-olive)]");
        }

        if show_result {
            let correct_answer = is_statement_correct;
            if !correct_answer {
                classes.push_str(" bg-[var(--success)] bg-opacity-20 border-[var(--success)]");
            } else if no_selected {
                classes.push_str(" bg-[var(--error)] bg-opacity-20 border-[var(--error)]");
            } else {
                classes.push_str(" opacity-50");
            }
        } else if no_selected {
            classes
                .push_str(" bg-[var(--accent-olive)] bg-opacity-10 border-[var(--accent-olive)]");
        }

        classes
    };

    let correct_answer_text = move || {
        if is_statement_correct {
            "Да"
        } else {
            "Нет"
        }
    };

    view! {
        <Card class=Signal::derive(|| "p-4 sm:p-6 min-h-[250px] sm:min-h-[300px] flex flex-col".to_string()) shadow=Signal::derive(|| true)>
            <QuizCardHeader
                card_type=card_type
                question_text=question_text.get_value()
            />

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-3 sm:mb-6">
                    <Show when=move || card_type != CardType::Kanji>
                        <div class="mb-4">
                            <Heading level=HeadingLevel::H2>
                                <FuriganaText text=statement.get_value() known_kanji=known_kanji.get()/>
                            </Heading>
                        </div>
                    </Show>

                    <Show when=move || kanji_for_animation.get_value().is_some()>
                        {move || {
                            let stmt = statement.get_value();
                            kanji_for_animation.get_value().map(|kanji: String| {
                                view! {
                                    <div class="mb-3 sm:mb-6">
                                        <DisplayText>
                                            {kanji}
                                        </DisplayText>
                                    </div>
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        {stmt}
                                    </Text>
                                }
                            })
                        }}
                    </Show>

                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                        "Верно ли это утверждение?"
                    </Text>
                </div>

                <div class="flex gap-3 sm:gap-4 justify-center">
                    <button
                        class=yes_class
                        disabled=show_result
                        on:click=move |_| {
                            if !show_result {
                                on_answer.run(true);
                            }
                        }
                    >
                        <Text size=TextSize::Large>
                            "Да"
                        </Text>
                    </button>
                    <button
                        class=no_class
                        disabled=show_result
                        on:click=move |_| {
                            if !show_result {
                                on_answer.run(false);
                            }
                        }
                    >
                        <Text size=TextSize::Large>
                            "Нет"
                        </Text>
                    </button>
                </div>

                <Show when=move || show_result>
                    <div class="mt-6 text-center">
                        <Text size=TextSize::Default class=move || yesno_result().result_class().to_string()>
                            {move || yesno_result().result_text()}
                        </Text>
                    </div>

                    <div class="mt-3 text-center">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {"Правильный ответ: "}
                            <span class="font-semibold">
                                {correct_answer_text}
                            </span>
                        </Text>
                    </div>
                </Show>
            </div>
        </Card>
    }
}
