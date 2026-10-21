use crate::i18n::*;
use crate::ui_components::{
    DisplayText, FuriganaText, Heading, HeadingLevel, KanjiViewMode, KanjiWritingSection,
    MarkdownText, MarkdownVariant, Text, TextSize, TypographyVariant, is_speech_supported,
    speak_word, stop_current_audio,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, MultiQuizResult, NativeLanguage, QuizCard, QuizMode};
use std::collections::HashSet;

use super::card_text::{card_question_text, display_question_text};
use super::card_type::CardType;
use super::lesson_card_header::{CardHeaderAudio, LessonCardTagsQuiz};
use super::lesson_card_shell::LessonCardShell;
use super::next_card_button::NextCardButton;
use super::quiz_options::QuizOptions;
use super::quiz_options_multi::QuizOptionsMulti;
use super::quiz_result::QuizResult;
use super::quiz_result_display::QuizResultDisplay;

#[derive(Clone, Copy, Default, PartialEq)]
pub enum QuizVariant {
    #[default]
    Meaning,
    Reading,
    Grammar,
}

/// Reading-quiz selections speak the chosen option (owner request):
/// interactive audio feedback for the reading the user picked. Other
/// variants stay silent — meaning/grammar options are native text, not
/// pronunciations. Muted lessons never speak.
fn should_speak_reading_selection(
    variant: QuizVariant,
    is_muted: bool,
    speech_supported: bool,
) -> bool {
    variant == QuizVariant::Reading && !is_muted && speech_supported
}

#[component]
pub fn QuizCardView(
    quiz_card: QuizCard,
    show_result: Signal<bool>,
    selected_option: Option<usize>,
    on_select_option: Callback<usize>,
    on_dont_know: Callback<()>,
    dont_know_selected: Signal<bool>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    #[prop(optional)] quiz_variant: QuizVariant,
    #[prop(into, default = Signal::derive(|| HashSet::new()))] selected_options: Signal<
        HashSet<usize>,
    >,
    #[prop(default = Signal::derive(|| false))] multi_submitted: Signal<bool>,
    #[prop(default = None)] multi_result: Option<MultiQuizResult>,
    #[prop(default = Callback::new(|_: usize| {}))] on_toggle: Callback<usize>,
    #[prop(default = Callback::new(|_: ()| {}))] on_submit: Callback<()>,
    #[prop(default = Signal::derive(|| false))] waiting_for_next: Signal<bool>,
    #[prop(default = Callback::new(|_: ()| {}))] on_next_card: Callback<()>,
    #[prop(default = false)] lenient_grading: bool,
    // Base word the grammar form was built from. Lesson container forwards
    // only QuizCard to this view, which shows the rule title as the question
    // and inflected forms as options — without this prop the base word is
    // invisible (#502). Shown only for QuizVariant::Grammar.
    #[prop(default = None)] grammar_base_word: Option<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card = quiz_card.card().clone();
    let card_type = CardType::from(&card);
    let lang = native_language;
    let part_of_speech = card.vocabulary_part_of_speech();

    let question_text = card_question_text(&card, &lang, card_type);
    let question = StoredValue::new(question_text.clone());
    let display_question = StoredValue::new(display_question_text(&card, &lang, card_type));
    let options: StoredValue<Vec<origa::domain::QuizOption>> =
        StoredValue::new(quiz_card.options().to_vec());
    let multi_result_stored = StoredValue::new(multi_result);
    let base_word_stored = StoredValue::new(grammar_base_word);

    // Kanji reading quiz answer side (#822): the answer must show the
    // kanji NAME (meaning), like every other answer side. `Card::Kanji`
    // answers with `CardAnswer::Text` (the name); a lookup failure must
    // not render an empty box — degrade with a warning instead.
    let kanji_name = kanji_quiz_answer_name(&card, &lang);
    let kanji_name_stored = StoredValue::new(kanji_name);

    let quiz_result = move || {
        if dont_know_selected.get() && show_result.get() {
            return QuizResult::DontKnow;
        }
        if let Some(selected) = selected_option {
            let opts = options.get_value();
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

    let kanji_for_animation: StoredValue<Option<String>> = StoredValue::new(match &card {
        DomainCard::Kanji(_) => Some(card_question_text(&card, &lang, card_type)),
        _ => None,
    });

    let lesson_ctx = use_context::<super::lesson_state::LessonContext>();
    let question_text = question.get_value();

    // Reading selections speak the picked option: single-select clicks,
    // multi-select toggles and keyboard [1..n] all funnel through the
    // wrapped callbacks below.
    let speak_selected_reading = {
        let lesson_ctx = lesson_ctx.clone();
        Callback::new(move |index: usize| {
            let is_muted = lesson_ctx
                .as_ref()
                .map(|ctx| ctx.is_muted.get_untracked())
                .unwrap_or(false);
            if !should_speak_reading_selection(quiz_variant, is_muted, is_speech_supported()) {
                return;
            }
            if let Some(option) = options.get_value().get(index) {
                speak_word(option.text(), 1.0);
            }
        })
    };
    let on_select_with_speech = {
        let speak = speak_selected_reading;
        Callback::new(move |index: usize| {
            speak.run(index);
            on_select_option.run(index);
        })
    };
    let on_toggle_with_speech = {
        let speak = speak_selected_reading;
        Callback::new(move |index: usize| {
            speak.run(index);
            on_toggle.run(index);
        })
    };

    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get_untracked())
            .unwrap_or(false);
        if !show_result.get() && card_type != CardType::Kanji && is_speech_supported() && !is_muted
        {
            speak_word(&question_text, 1.0);
        }
    });

    on_cleanup(move || {
        stop_current_audio();
    });

    let is_multi = quiz_card.mode() == QuizMode::Multi;

    let tags_row: ChildrenFn = std::sync::Arc::new(move || {
        view! {
                <LessonCardTagsQuiz
                    card_type=card_type
                    quiz_variant=quiz_variant
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

    view! {
        <LessonCardShell
            tags=tags_row
            test_id="lesson-card-root"
        >
        <div class="text-center mb-0.5 sm:mb-1">
                <Show when=move || kanji_for_animation.get_value().is_none()>
                    <div class="mb-4">
                        <Heading level=HeadingLevel::H2>
                            <FuriganaText text=display_question.get_value() known_kanji=known_kanji.get() native_language=native_language with_kanji_tooltip=true/>
                        </Heading>
                    </div>
                    // Grammar quizzes conjugate a base word that is not part of
                    // the question (title) or the options (inflected forms).
                    // Show it explicitly, mirroring the practice session (#502).
                    <Show when=move || {
                        quiz_variant == QuizVariant::Grammar
                            && base_word_stored.get_value().is_some_and(|s| !s.is_empty())
                    }>
                        <div
                            class="mb-4 font-serif text-2xl text-[var(--fg-black)]"
                            data-testid="lesson-grammar-base-word"
                        >
                            <FuriganaText
                                text=base_word_stored.get_value().unwrap_or_default()
                                known_kanji=known_kanji.get()
                                native_language=native_language
                            />
                        </div>
                    </Show>
                </Show>

                <Show when=move || kanji_for_animation.get_value().is_some()>
                    {move || {
                        kanji_for_animation.get_value().map(|kanji: String| {
                            let kanji_clone = kanji.clone();
                            view! {
                                <div class="mb-3 sm:mb-6">
                                    <DisplayText>
                                        {kanji}
                                    </DisplayText>
                                </div>
                                <KanjiWritingSection kanji=kanji_clone mode=KanjiViewMode::Animation />
                            }
                        })
                    }}
                </Show>

                <Show when=move || !show_result.get()>
                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                        {if is_multi && quiz_variant == QuizVariant::Reading {
                            t!(i18n, lesson.choose_all_readings).into_any()
                        } else {
                            match quiz_variant {
                                QuizVariant::Meaning => t!(i18n, lesson.choose_answer).into_any(),
                                QuizVariant::Reading => t!(i18n, lesson.choose_reading).into_any(),
                                QuizVariant::Grammar => t!(i18n, lesson.choose_grammar).into_any(),
                            }
                        }}
                    </Text>
                </Show>
            </div>

            <Show when=move || is_multi>
                <QuizOptionsMulti
                    options=options.get_value()
                    selected_options=selected_options.get()
                    show_result=show_result
                    multi_submitted=multi_submitted
                    multi_result=multi_result_stored.get_value()
                    on_toggle=on_toggle_with_speech
                    on_submit=on_submit
                    on_dont_know=on_dont_know
                    dont_know_selected=dont_know_selected
                    known_kanji=known_kanji
                    waiting_for_next=waiting_for_next
                    on_next_card=on_next_card
                />
                <Show when=move || multi_submitted.get()>
                    {move || {
                        multi_result_stored
                            .get_value()
                            .map(|r| {
                                let result = if lenient_grading {
                                    QuizResult::from_multi_result_lenient(&r)
                                } else {
                                    QuizResult::from_multi_result(&r)
                                };
                                view! {
                                    <QuizResultDisplay
                                        quiz_result=result
                                        multi_result=multi_result_stored.get_value()
                                        options=options.get_value()
                                    />
                                }
                            })
                    }}
                </Show>
                // The kanji NAME on the answer side (#822): same secondary
                // box style as the single-quiz correct-answer reveal.
                <Show
                    when=move || {
                        multi_submitted.get() && !kanji_name_stored.get_value().is_empty()
                    }
                >
                    <KanjiQuizAnswerName name=kanji_name_stored.get_value() />
                </Show>
            </Show>

            <Show when=move || !is_multi>
                <Show when=move || !show_result.get()>
                    <QuizOptions
                        options=options.get_value()
                        selected_option=selected_option
                        show_result=show_result
                        quiz_result=quiz_result()
                        on_select_option=on_select_with_speech
                        on_dont_know=on_dont_know
                        dont_know_selected=dont_know_selected
                        known_kanji=known_kanji
                    />
                </Show>

                <Show when=move || show_result.get()>
                    <QuizResultDisplay quiz_result=quiz_result() />

                    // Show the correct answer so the user can learn from
                    // mistakes or confirm their choice.
                    <div class="mt-3 p-3 bg-[var(--bg-secondary)] text-center">
                        <MarkdownText
                            content=Signal::derive(move || {
                                options
                                    .get_value()
                                    .iter()
                                    .find(|o| o.is_correct())
                                    .map(|o| o.text().to_string())
                                    .unwrap_or_default()
                            })
                            variant=MarkdownVariant::Default
                            known_kanji=known_kanji.get()
                        />
                    </div>
                </Show>

                <Show when=move || waiting_for_next.get() && show_result.get()>
                    <NextCardButton on_next_card=on_next_card />
                </Show>
            </Show>
        </LessonCardShell>
    }
}

/// The kanji NAME box on the reading-quiz answer side (#822): muted
/// «Значение» label + the name, styled like the single-quiz
/// correct-answer reveal. Presentational only — the caller gates it on
/// submission and a non-empty name.
#[component]
pub(super) fn KanjiQuizAnswerName(name: String) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <div class="mt-3 p-3 bg-[var(--bg-secondary)] text-center">
            <Text size=TextSize::Default variant=TypographyVariant::Muted class="mb-1 block">
                {t!(i18n, lesson.meaning)}
            </Text>
            <Text size=TextSize::Large class="text-primary">
                {name}
            </Text>
        </div>
    }
}

/// Resolve the kanji NAME for the reading-quiz answer side (#822).
/// `Card::Kanji` answers with `CardAnswer::Text` (the name); other card
/// kinds have no name here and a lookup failure degrades to an empty
/// string — with a warning, so a broken dictionary stays visible in
/// logs instead of silently reproducing the «no kanji name» bug.
fn kanji_quiz_answer_name(card: &DomainCard, lang: &NativeLanguage) -> String {
    match card.answer(lang) {
        Ok(origa::domain::CardAnswer::Text(name)) => name,
        Ok(_) => String::new(),
        Err(e) => {
            tracing::warn!(
                error = %e,
                content_key = %card.content_key(),
                "Failed to resolve kanji name for reading quiz answer"
            );
            String::new()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::reading_unmuted_supported(QuizVariant::Reading, false, true, true)]
    #[case::muted_lesson_never_speaks(QuizVariant::Reading, true, true, false)]
    #[case::no_tts_environment(QuizVariant::Reading, false, false, false)]
    #[case::meaning_variant_stays_silent(QuizVariant::Meaning, false, true, false)]
    #[case::grammar_variant_stays_silent(QuizVariant::Grammar, false, true, false)]
    fn reading_selection_speaks_only_for_reading_quizzes_in_unmuted_tts_env(
        #[case] variant: QuizVariant,
        #[case] is_muted: bool,
        #[case] speech_supported: bool,
        #[case] expected: bool,
    ) {
        assert_eq!(
            should_speak_reading_selection(variant, is_muted, speech_supported),
            expected
        );
    }

    /// Non-kanji cards answer with a structured variant (no
    /// `CardAnswer::Text`), so the name box must stay hidden — the
    /// multi name box exists only for kanji reading quizzes (#822).
    #[test]
    fn non_kanji_card_yields_empty_quiz_answer_name() {
        let card: DomainCard = serde_json::from_str(
            r#"{"Vocabulary":{"word":{"text":"test"},"reverse_side":null,"pos":null}}"#,
        )
        .expect("deserialize vocab card");
        assert!(kanji_quiz_answer_name(&card, &NativeLanguage::Russian).is_empty());
    }
}
