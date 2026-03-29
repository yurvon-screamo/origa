use crate::ui_components::{Card, get_reading_from_text, is_speech_supported, speak_text};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, GrammarInfo, NativeLanguage};
use std::collections::HashSet;
use tracing;

use super::card_type::CardType;
use super::kanji_card_details::RadicalDisplay;
use super::lesson_card_answer::LessonCardAnswer;
use super::lesson_card_header::LessonCardHeader;
use super::lesson_card_question::LessonCardQuestion;
use super::lesson_state::LessonContext;
use super::radical_card_details::RadicalCardDisplay;

#[component]
pub fn LessonCard(
    card: DomainCard,
    show_answer: bool,
    is_reversed: bool,
    on_show_answer: Callback<()>,
    grammar_info: Option<GrammarInfo>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<String>>,
) -> impl IntoView {
    let card_type = CardType::from(&card);
    let lang = native_language;
    let question = StoredValue::new(
        card.question(&lang)
            .ok()
            .map(|q| q.text().to_string())
            .unwrap_or_default(),
    );
    let answer = StoredValue::new(
        card.answer(&lang)
            .ok()
            .map(|a| a.text().to_string())
            .unwrap_or_default(),
    );

    let radicals: Option<Vec<RadicalDisplay>> = match &card {
        DomainCard::Kanji(kanji) => match kanji.radicals_info() {
            Ok(r) => Some(
                r.iter()
                    .map(|info| RadicalDisplay {
                        symbol: info.radical(),
                        name: info.name().to_string(),
                        description: info.description().to_string(),
                    })
                    .collect(),
            ),
            Err(e) => {
                tracing::warn!("Failed to get radicals for kanji: {:?}", e);
                None
            },
        },
        _ => None,
    };
    let radicals_stored = StoredValue::new(radicals);

    let example_words: Option<Vec<(String, String)>> = match &card {
        DomainCard::Kanji(kanji) => {
            let examples: Vec<_> = kanji
                .example_words(&lang)
                .iter()
                .map(|e| (e.word().to_string(), e.meaning().to_string()))
                .collect();
            if examples.is_empty() {
                None
            } else {
                Some(examples)
            }
        },
        _ => None,
    };
    let examples_stored = StoredValue::new(example_words);

    let on_readings: Option<Vec<String>> = match &card {
        DomainCard::Kanji(kanji) => {
            let readings = kanji.on_readings().to_vec();
            if readings.is_empty() {
                None
            } else {
                Some(readings)
            }
        },
        _ => None,
    };
    let on_readings_stored = StoredValue::new(on_readings);

    let kun_readings: Option<Vec<String>> = match &card {
        DomainCard::Kanji(kanji) => {
            let readings = kanji.kun_readings().to_vec();
            if readings.is_empty() {
                None
            } else {
                Some(readings)
            }
        },
        _ => None,
    };
    let kun_readings_stored = StoredValue::new(kun_readings);

    let kanji_for_animation: Option<String> = match &card {
        DomainCard::Kanji(_) | DomainCard::Radical(_) => Some(
            card.question(&lang)
                .ok()
                .map(|q| q.text().to_string())
                .unwrap_or_default(),
        ),
        _ => None,
    };
    let kanji_stored = StoredValue::new(kanji_for_animation);

    let radical_display: Option<RadicalCardDisplay> = match &card {
        DomainCard::Radical(radical) => match radical.radical_info() {
            Ok(info) => Some(RadicalCardDisplay {
                symbol: info.radical(),
                name: info.name().to_string(),
                description: info.description().to_string(),
                stroke_count: info.stroke_count(),
                kanji_examples: info.kanji().to_vec(),
            }),
            Err(e) => {
                tracing::warn!("Failed to get radical info: {:?}", e);
                None
            },
        },
        _ => None,
    };
    let radical_stored = StoredValue::new(radical_display);

    let lesson_ctx = use_context::<LessonContext>();

    let is_expanded = RwSignal::new(card_type == CardType::Kanji || card_type == CardType::Radical);
    let content_ref = NodeRef::<leptos::html::Div>::new();
    let needs_collapse = RwSignal::new(false);

    let lesson_ctx_tts_normal = lesson_ctx.clone();
    Effect::new(move |_| {
        let is_muted = lesson_ctx_tts_normal
            .as_ref()
            .map(|ctx| ctx.is_muted.get())
            .unwrap_or(false);
        if !show_answer
            && !is_reversed
            && card_type != CardType::Kanji
            && card_type != CardType::Radical
            && is_speech_supported()
            && !is_muted
        {
            let reading = get_reading_from_text(&question.get_value());
            let _ = speak_text(&reading, 1.0);
        }
    });

    let lesson_ctx_tts_reversed = lesson_ctx.clone();
    Effect::new(move |_| {
        let is_muted = lesson_ctx_tts_reversed
            .as_ref()
            .map(|ctx| ctx.is_muted.get())
            .unwrap_or(false);
        if show_answer
            && is_reversed
            && card_type != CardType::Kanji
            && card_type != CardType::Radical
            && is_speech_supported()
            && !is_muted
        {
            let reading = get_reading_from_text(&answer.get_value());
            let _ = speak_text(&reading, 1.0);
        }
    });

    Effect::new(move |_| {
        if show_answer && let Some(el) = content_ref.get() {
            let is_overflow = el.scroll_height() > el.client_height();
            needs_collapse.set(is_overflow);
        }
    });

    let on_toggle = Callback::new(move |()| {
        is_expanded.update(|v| *v = !*v);
    });

    view! {
        <Card class=Signal::derive(|| "p-4 sm:p-6 min-h-[250px] sm:min-h-[300px] flex flex-col".to_string()) shadow=Signal::derive(|| true)>
            <LessonCardHeader
                card_type=card_type
                question_text=if is_reversed { answer.get_value() } else { question.get_value() }
                grammar_info=grammar_info.clone()
                show_answer=show_answer
            />

            <div class="flex-1 flex flex-col justify-center">
                <Show when=move || !show_answer>
                    <LessonCardQuestion
                        question_text=question.get_value()
                        kanji=kanji_stored.get_value()
                        is_reversed=is_reversed
                        on_show_answer=on_show_answer
                        known_kanji=known_kanji
                    />
                </Show>

                <Show when=move || show_answer>
                    <LessonCardAnswer
                        question_text=question.get_value()
                        answer_text=answer.get_value()
                        is_expanded=is_expanded
                        needs_collapse=needs_collapse
                        content_ref=content_ref
                        on_toggle=on_toggle
                        is_kanji=card_type == CardType::Kanji
                        is_radical=card_type == CardType::Radical
                        is_reversed=is_reversed
                        on_readings=on_readings_stored.get_value()
                        kun_readings=kun_readings_stored.get_value()
                        radicals=radicals_stored.get_value()
                        radical=radical_stored.get_value()
                        example_words=examples_stored.get_value()
                        grammar_info=grammar_info.clone()
                        known_kanji=known_kanji
                    />
                </Show>
            </div>
        </Card>
    }
}
