use crate::i18n::*;
use crate::ui_components::{
    Card, DisplayText, FuriganaText, Heading, HeadingLevel, KanjiViewMode, KanjiWritingSection,
    Text, TextSize, TypographyVariant, get_reading_from_text, is_speech_supported, speak_text,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, QuizCard};
use std::collections::HashSet;

use super::card_type::CardType;
use super::quiz_card_header::QuizCardHeader;
use super::quiz_options::QuizOptions;
use super::quiz_result::QuizResult;
use super::quiz_result_display::QuizResultDisplay;

#[component]
pub fn QuizCardView(
    quiz_card: QuizCard,
    show_result: bool,
    selected_option: Option<usize>,
    on_select_option: Callback<usize>,
    on_dont_know: Callback<()>,
    dont_know_selected: bool,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<String>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card = quiz_card.card().clone();
    let card_type = CardType::from(&card);
    let lang = native_language;
    let question = StoredValue::new(
        card.question(&lang)
            .ok()
            .map(|q| q.text().to_string())
            .unwrap_or_default(),
    );
    let options: StoredValue<Vec<origa::domain::QuizOption>> =
        StoredValue::new(quiz_card.options().to_vec());

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
        DomainCard::Kanji(_) => Some(
            card.question(&lang)
                .ok()
                .map(|q| q.text().to_string())
                .unwrap_or_default(),
        ),
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
            let reading = get_reading_from_text(&question_text);
            let _ = speak_text(&reading, 1.0);
        }
    });

    view! {
        <Card class=Signal::derive(|| "p-4 sm:p-6 min-h-[250px] sm:min-h-[300px] flex flex-col".to_string()) shadow=Signal::derive(|| true)>
            <QuizCardHeader
                card_type=card_type
                question_text=question.get_value()
            />

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-3 sm:mb-6">
                    <Show when=move || kanji_for_animation.get_value().is_none()>
                        <div class="mb-4">
                            <Heading level=HeadingLevel::H2>
                                <FuriganaText text=question.get_value() known_kanji=known_kanji.get()/>
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
                        {t!(i18n, lesson.choose_answer)}
                    </Text>
                </div>

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
            </div>
        </Card>
    }
}
