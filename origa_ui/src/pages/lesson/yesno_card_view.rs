use crate::i18n::*;
use crate::ui_components::{
    Button, ButtonVariant, Card, DisplayText, MarkdownText, MarkdownVariant, Text, TextSize,
    TypographyVariant, get_reading_from_text, is_speech_supported, speak_text,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, YesNoCard};
use std::collections::HashSet;
use tracing::warn;

use super::card_type::CardType;
use super::quiz_card_header::QuizCardHeader;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum YesNoResult {
    #[default]
    None,
    Correct,
    Incorrect,
    DontKnow,
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
}

#[component]
pub fn YesNoCardView(
    yesno_card: YesNoCard,
    show_result: bool,
    selected_answer: Option<bool>,
    on_answer: Callback<bool>,
    on_dont_know: Callback<()>,
    dont_know_selected: bool,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<String>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card = yesno_card.card().clone();
    let card_type = CardType::from(&card);
    let lang = native_language;
    let statement = StoredValue::new(yesno_card.statement_text().to_string());
    let is_statement_correct = yesno_card.is_correct();

    let question_text = StoredValue::new(match card.question(&lang) {
        Ok(q) => q.text().to_string(),
        Err(e) => {
            warn!(
                card_type = ?card_type,
                content_key = %card.content_key(),
                error = %e,
                "Failed to get card question"
            );
            String::new()
        },
    });

    let kanji_for_animation: StoredValue<Option<String>> = StoredValue::new(match &card {
        DomainCard::Kanji(_) => {
            let text = match card.question(&lang) {
                Ok(q) => q.text().to_string(),
                Err(e) => {
                    warn!(
                        card_type = ?card_type,
                        content_key = %card.content_key(),
                        error = %e,
                        "Failed to get card question"
                    );
                    String::new()
                },
            };
            Some(text)
        },
        _ => None,
    });

    let lesson_ctx = use_context::<super::lesson_state::LessonContext>();
    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get())
            .unwrap_or(false);
        if !show_result && card_type != CardType::Kanji && is_speech_supported() && !is_muted {
            let reading = get_reading_from_text(&question_text.get_value());
            let _ = speak_text(&reading, 1.0);
        }
    });

    let yesno_result = move || {
        if dont_know_selected && show_result {
            return YesNoResult::DontKnow;
        }
        YesNoResult::from_answer(
            is_statement_correct,
            selected_answer.unwrap_or(false),
            show_result,
        )
    };

    let yes_selected = selected_answer == Some(true);
    let no_selected = selected_answer == Some(false);

    let no_btn_class = Signal::derive(move || {
        if show_result {
            if !is_statement_correct {
                "quiz-option-correct".to_string()
            } else if no_selected {
                "quiz-option-wrong".to_string()
            } else {
                "quiz-option-dimmed".to_string()
            }
        } else {
            String::new()
        }
    });

    let yes_btn_class = Signal::derive(move || {
        if show_result {
            if is_statement_correct {
                "quiz-option-correct".to_string()
            } else if yes_selected {
                "quiz-option-wrong".to_string()
            } else {
                "quiz-option-dimmed".to_string()
            }
        } else {
            String::new()
        }
    });

    let yes_variant = Signal::derive(move || {
        if show_result {
            ButtonVariant::Default
        } else {
            ButtonVariant::Olive
        }
    });

    let correct_answer_text = move || {
        if is_statement_correct {
            i18n.get_keys().lesson().yes().inner().to_string()
        } else {
            i18n.get_keys().lesson().no().inner().to_string()
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
                    <Show when=move || kanji_for_animation.get_value().is_none()>
                        <div class="mb-4">
                            <MarkdownText
                                content=Signal::derive(move || statement.get_value())
                                known_kanji=known_kanji.get()
                                variant=Signal::derive(|| MarkdownVariant::Large)
                            />
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
                        {t!(i18n, lesson.is_this_correct)}
                    </Text>
                </div>

                <div class="grid grid-cols-2 gap-3">
                    <Button
                        test_id=Signal::derive(|| "yesno-no-btn".to_string())
                        variant=Signal::derive(|| ButtonVariant::Default)
                        class=no_btn_class
                        disabled=Signal::derive(move || show_result)
                        on_click=Callback::new(move |_| on_answer.run(false))
                    >
                        {t!(i18n, lesson.no)} <span class="hidden sm:inline">"[1]"</span>
                    </Button>

                    <Button
                        test_id=Signal::derive(|| "yesno-yes-btn".to_string())
                        variant=yes_variant
                        class=yes_btn_class
                        disabled=Signal::derive(move || show_result)
                        on_click=Callback::new(move |_| on_answer.run(true))
                    >
                        {t!(i18n, lesson.yes)} <span class="hidden sm:inline">"[2]"</span>
                    </Button>
                </div>
                <button
                    data-testid="yesno-dont-know-btn"
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
                    <span class="hidden sm:inline text-[var(--fg-muted)] text-xs font-mono">{t!(i18n, lesson.space_key)}</span>
                </button>

                <Show when=move || show_result>
                    <Show when=move || yesno_result() == YesNoResult::Correct>
                        <div class="mt-6 text-center">
                            <Text size=TextSize::Default class="text-[var(--success)] font-bold">
                                {t!(i18n, lesson.correct)}
                            </Text>
                        </div>
                    </Show>

                    <Show when=move || matches!(yesno_result(), YesNoResult::Incorrect)>
                        <div class="mt-6 text-center">
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {t!(i18n, lesson.correct_answer)}
                                <span class="font-semibold">
                                    {correct_answer_text}
                                </span>
                            </Text>
                        </div>
                    </Show>

                    <Show when=move || yesno_result() == YesNoResult::DontKnow>
                        <div class="mt-6 text-center">
                            <Text size=TextSize::Default class="text-[var(--fg-muted)] font-bold">
                                {t!(i18n, lesson.dont_know_result)}
                            </Text>
                        </div>
                    </Show>
                </Show>
            </div>
        </Card>
    }
}
