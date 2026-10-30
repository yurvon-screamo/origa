use crate::i18n::*;
use crate::ui_components::{
    Button, ButtonVariant, DisplayText, MarkdownText, MarkdownVariant, Text, TextSize,
    TypographyVariant, is_speech_supported, speak_word, stop_current_audio,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, YesNoCard};
use std::collections::HashSet;

use super::answer_display::{CardAnswerDisplay, extract_card_answer};
use super::card_text::{card_question_text, display_question_text};
use super::card_type::CardType;
use super::dont_know_button::DontKnowButton;
use super::lesson_card_header::{CardHeaderAudio, LessonCardTagsQuiz};
use super::lesson_card_shell::LessonCardShell;
use super::next_card_button::NextCardButton;
use super::quiz_feedback_strip::QuizFeedbackStrip;
use super::quiz_ledger::{single_verdict, yesno_ledger_rows};
use super::quiz_ledger_labels::yesno_label;
use super::quiz_result::QuizResult;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_answer_before_result_is_always_none() {
        assert_eq!(
            YesNoResult::from_answer(true, true, false),
            YesNoResult::None
        );
    }

    #[test]
    fn from_answer_correct_statement_yes_is_correct() {
        assert_eq!(
            YesNoResult::from_answer(true, true, true),
            YesNoResult::Correct
        );
    }

    #[test]
    fn from_answer_correct_statement_no_is_incorrect() {
        assert_eq!(
            YesNoResult::from_answer(true, false, true),
            YesNoResult::Incorrect
        );
    }

    #[test]
    fn from_answer_wrong_statement_no_is_correct() {
        assert_eq!(
            YesNoResult::from_answer(false, false, true),
            YesNoResult::Correct
        );
    }

    #[test]
    fn from_answer_wrong_statement_yes_is_incorrect() {
        assert_eq!(
            YesNoResult::from_answer(false, true, true),
            YesNoResult::Incorrect
        );
    }
}

#[component]
pub fn YesNoCardView(
    yesno_card: YesNoCard,
    show_result: Signal<bool>,
    selected_answer: Option<bool>,
    on_answer: Callback<bool>,
    on_dont_know: Callback<()>,
    dont_know_selected: Signal<bool>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    #[prop(default = Signal::derive(|| false))] waiting_for_next: Signal<bool>,
    #[prop(default = Callback::new(|_: ()| {}))] on_next_card: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card = yesno_card.card().clone();
    let card_type = CardType::from(&card);
    let lang = native_language;
    let part_of_speech = card.vocabulary_part_of_speech();

    let word_raw = yesno_card.word().to_string();
    let display_word =
        StoredValue::new(if super::na_adjective_helper::is_na_adjective_card(&card) {
            super::na_adjective_helper::append_na_suffix(&word_raw)
        } else {
            word_raw
        });
    let statement_value = StoredValue::new(yesno_card.statement().to_string());
    let is_statement_correct = yesno_card.is_correct();

    let question_text = StoredValue::new(card_question_text(&card, &lang, card_type));
    let display_question = StoredValue::new(display_question_text(&card, &lang, card_type));

    let kanji_for_animation: StoredValue<Option<String>> = StoredValue::new(match &card {
        DomainCard::Kanji(_) => Some(card_question_text(&card, &lang, card_type)),
        _ => None,
    });

    let answer_data = extract_card_answer(&card, &native_language, &card_type);
    let answer_vocab_translations_stored = StoredValue::new(answer_data.translations);
    let answer_vocab_description_stored = StoredValue::new(answer_data.description);
    let answer_text_display_stored = StoredValue::new(answer_data.text);

    let lesson_ctx = use_context::<super::lesson_state::LessonContext>();
    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get_untracked())
            .unwrap_or(false);
        if !show_result.get() && card_type != CardType::Kanji && is_speech_supported() && !is_muted
        {
            speak_word(&question_text.get_value(), 1.0);
        }
    });

    on_cleanup(move || {
        stop_current_audio();
    });

    let yesno_result = move || {
        if dont_know_selected.get() && show_result.get() {
            return YesNoResult::DontKnow;
        }
        YesNoResult::from_answer(
            is_statement_correct,
            selected_answer.unwrap_or(false),
            show_result.get(),
        )
    };

    let tags_row: ChildrenFn = std::sync::Arc::new(move || {
        view! {
                <LessonCardTagsQuiz
                    card_type=card_type
                    part_of_speech=part_of_speech.clone()
                    audio=Signal::derive(move || Some(CardHeaderAudio {
                        card_type,
                        question_text: display_question.get_value(),
                        is_reversed: false,
                        audio_path: None,
                    }))
                />
        }
        .into_any()
    });

    // Feedback strip below the card (direction B, task 3SekkMiYYW):
    // verdict + Да/Нет ledger rows. Replaces the old in-card verdict
    // text + «Correct answer: Да/Нет» line.
    let feedback_row: ChildrenFn = std::sync::Arc::new(move || {
        view! {
            <Show when=move || show_result.get()>
                {move || {
                    let result = QuizResult::from(yesno_result());
                    let verdict = single_verdict(result)?;
                    let rows = yesno_ledger_rows(
                        result,
                        selected_answer,
                        is_statement_correct,
                        &yesno_label(i18n, true),
                        &yesno_label(i18n, false),
                    );
                    Some(view! {
                        <QuizFeedbackStrip verdict rows known_kanji />
                    })
                }}
            </Show>
        }
        .into_any()
    });

    // Action footer: Да/Нет + «не знаю» on the question side, the
    // next-card button on the result side.
    let footer_row: ChildrenFn = std::sync::Arc::new(move || {
        view! {
            <Show when=move || !show_result.get()>
                <div class="grid grid-cols-2 gap-3">
                    <Button
                        test_id=Signal::derive(|| "yesno-no-btn".to_string())
                        variant=Signal::derive(|| ButtonVariant::Default)
                        disabled=Signal::derive(move || show_result.get())
                        on_click=Callback::new(move |_| on_answer.run(false))
                    >
                        {t!(i18n, lesson.no)} <span class="kbd-hint">"[1]"</span>
                    </Button>

                    <Button
                        test_id=Signal::derive(|| "yesno-yes-btn".to_string())
                        variant=Signal::derive(|| ButtonVariant::Olive)
                        disabled=Signal::derive(move || show_result.get())
                        on_click=Callback::new(move |_| on_answer.run(true))
                    >
                        {t!(i18n, lesson.yes)} <span class="kbd-hint">"[2]"</span>
                    </Button>
                </div>
                <DontKnowButton
                    dont_know_selected
                    disabled=show_result
                    test_id="yesno-dont-know-btn"
                    on_click=on_dont_know
                />
            </Show>

            <Show when=move || waiting_for_next.get() && show_result.get()>
                <NextCardButton on_next_card=on_next_card />
            </Show>
        }
        .into_any()
    });

    view! {
        <LessonCardShell
            tags=tags_row
            feedback=feedback_row
            footer=footer_row
            test_id="lesson-card-root"
        >
        <div class="text-center mb-3 sm:mb-6">
                <Show when=move || kanji_for_animation.get_value().is_none()>
                    <div class="mb-2 sm:mb-3">
                        <MarkdownText
                            content=Signal::derive(move || display_word.get_value())
                            known_kanji=known_kanji.get()
                            variant=Signal::derive(|| MarkdownVariant::Large)
                        />
                    </div>
                </Show>

                <Show when=move || kanji_for_animation.get_value().is_some()>
                    {move || {
                        kanji_for_animation.get_value().map(|kanji: String| {
                            view! {
                                <div class="mb-2 sm:mb-3">
                                    <DisplayText>
                                        {kanji}
                                    </DisplayText>
                                </div>
                            }
                        })
                    }}
                </Show>

                // The statement (the claimed translation) is the
                // question. On the result side a CORRECT claim would
                // duplicate the revealed answer below the verdict —
                // hide it there; a distractor claim stays visible as
                // the contrast against the real answer.
                <Show when=move || !show_result.get() || !is_statement_correct>
                    <div data-testid="yesno-statement">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            {move || statement_value.get_value()}
                        </Text>
                    </div>
                </Show>

                <Show when=move || !show_result.get()>
                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                        {t!(i18n, lesson.is_this_correct)}
                    </Text>
                </Show>
            </div>

            // The yes/no verdict alone is not the answer the learner
            // wants — the actual card answer (translation / meaning /
            // description) is revealed on every result, correct or not.
            <Show when=move || show_result.get()>
                <CardAnswerDisplay
                    translations=Signal::derive(move || answer_vocab_translations_stored.get_value())
                    description=Signal::derive(move || answer_vocab_description_stored.get_value())
                    text=Signal::derive(move || answer_text_display_stored.get_value())
                    known_kanji=known_kanji
                />
            </Show>
        </LessonCardShell>
    }
}
