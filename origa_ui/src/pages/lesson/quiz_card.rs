use crate::i18n::*;
use crate::ui_components::{
    Card, DisplayText, FuriganaText, Heading, HeadingLevel, KanjiViewMode, KanjiWritingSection,
    Text, TextSize, TypographyVariant, is_speech_supported, speak_word, stop_current_audio,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, MultiQuizResult, NativeLanguage, QuizCard, QuizMode};
use std::collections::HashSet;
use tracing::warn;

use super::card_type::CardType;
use super::quiz_card_header::QuizCardHeader;
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
    Radicals,
}

#[component]
pub fn QuizCardView(
    quiz_card: QuizCard,
    show_result: bool,
    selected_option: Option<usize>,
    on_select_option: Callback<usize>,
    on_dont_know: Callback<()>,
    dont_know_selected: bool,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    #[prop(optional)] quiz_variant: QuizVariant,
    #[prop(into, default = Signal::derive(|| HashSet::new()))] selected_options: Signal<
        HashSet<usize>,
    >,
    #[prop(default = false)] multi_submitted: bool,
    #[prop(default = None)] multi_result: Option<MultiQuizResult>,
    #[prop(default = Callback::new(|_: usize| {}))] on_toggle: Callback<usize>,
    #[prop(default = Callback::new(|_: ()| {}))] on_submit: Callback<()>,
    #[prop(default = false)] waiting_for_next: bool,
    #[prop(default = Callback::new(|_: ()| {}))] on_next_card: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card = quiz_card.card().clone();
    let card_type = CardType::from(&card);
    let lang = native_language;
    let is_na_adj = super::na_adjective_helper::is_na_adjective_card(&card);

    let question_text = match card.question(&lang) {
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
    let question = StoredValue::new(question_text.clone());
    let display_question = StoredValue::new(if is_na_adj {
        super::na_adjective_helper::append_na_suffix(&question_text)
    } else {
        question_text.clone()
    });
    let options: StoredValue<Vec<origa::domain::QuizOption>> =
        StoredValue::new(quiz_card.options().to_vec());
    let multi_result_stored = StoredValue::new(multi_result);

    let quiz_result = move || {
        if dont_know_selected && show_result {
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
    let question_text = question.get_value();

    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get())
            .unwrap_or(false);
        if !show_result && card_type != CardType::Kanji && is_speech_supported() && !is_muted {
            speak_word(&question_text, 1.0);
        }
    });

    on_cleanup(move || {
        stop_current_audio();
    });

    let is_multi = quiz_card.mode() == QuizMode::Multi;

    view! {
        <Card class=Signal::derive(|| "p-4 sm:p-6 min-h-[250px] sm:min-h-[300px] flex flex-col".to_string()) shadow=Signal::derive(|| true)>
            <QuizCardHeader
                card_type=card_type
                question_text=display_question.get_value()
                quiz_variant=quiz_variant
            />

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-3 sm:mb-6">
                    <Show when=move || kanji_for_animation.get_value().is_none()>
                        <div class="mb-4">
                            <Heading level=HeadingLevel::H2>
                                <FuriganaText text=display_question.get_value() known_kanji=known_kanji.get()/>
                            </Heading>
                        </div>
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

                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                        {if is_multi && quiz_variant == QuizVariant::Reading {
                            t!(i18n, lesson.choose_all_readings).into_any()
                        } else if is_multi && quiz_variant == QuizVariant::Radicals {
                            t!(i18n, lesson.choose_all_radicals).into_any()
                        } else {
                            match quiz_variant {
                                QuizVariant::Meaning => t!(i18n, lesson.choose_answer).into_any(),
                                QuizVariant::Reading => t!(i18n, lesson.choose_reading).into_any(),
                                QuizVariant::Grammar => t!(i18n, lesson.choose_grammar).into_any(),
                                QuizVariant::Radicals => t!(i18n, lesson.choose_all_radicals).into_any(),
                            }
                        }}
                    </Text>
                </div>

                <Show when=move || is_multi>
                    <QuizOptionsMulti
                        options=options.get_value()
                        selected_options=selected_options.get()
                        show_result=show_result
                        multi_submitted=multi_submitted
                        multi_result=multi_result_stored.get_value()
                        on_toggle=on_toggle
                        on_submit=on_submit
                        on_dont_know=on_dont_know
                        dont_know_selected=dont_know_selected
                        known_kanji=known_kanji
                        waiting_for_next=waiting_for_next
                        on_next_card=on_next_card
                    />
                    <Show when=move || multi_submitted>
                        {move || {
                            multi_result_stored
                                .get_value()
                                .map(|r| {
                                    let result = QuizResult::from_multi_result(&r);
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
                </Show>

                <Show when=move || !is_multi>
                    <QuizOptions
                        options=options.get_value()
                        selected_option=selected_option
                        show_result=show_result
                        quiz_result=quiz_result()
                        on_select_option=on_select_option
                        on_dont_know=on_dont_know
                        dont_know_selected=dont_know_selected
                        known_kanji=known_kanji
                    />

                    <Show when=move || show_result && quiz_result() != QuizResult::DontKnow>
                        <QuizResultDisplay quiz_result=quiz_result() />
                    </Show>
                </Show>
            </div>
        </Card>
    }
}
