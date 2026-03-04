use crate::ui_components::{Card, get_reading_from_text, is_speech_supported, speak_text};
use leptos::prelude::*;
use origa::domain::Card as DomainCard;

use super::card_type::CardType;
use super::kanji_card_details::KanjiCardDetails;
use super::lesson_card_answer::LessonCardAnswer;
use super::lesson_card_header::LessonCardHeader;
use super::lesson_card_question::LessonCardQuestion;
use super::lesson_state::LessonContext;

#[component]
pub fn LessonCard(
    card: DomainCard,
    show_answer: bool,
    is_reversed: bool,
    on_show_answer: Callback<()>,
) -> impl IntoView {
    let card_type = CardType::from(&card);
    let question = StoredValue::new(card.question().text().to_string());
    let answer = StoredValue::new(card.answer().text().to_string());

    let radicals: Option<String> = match &card {
        DomainCard::Kanji(kanji) => kanji.radicals_info().ok().map(|r| {
            r.iter()
                .map(|info| info.radical().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        }),
        _ => None,
    };
    let radicals_stored = StoredValue::new(radicals);

    let example_words: Option<Vec<(String, String)>> = match &card {
        DomainCard::Kanji(kanji) => {
            let examples: Vec<_> = kanji
                .example_words()
                .iter()
                .map(|e| (e.word().to_string(), e.meaning().to_string()))
                .collect();
            if examples.is_empty() {
                None
            } else {
                Some(examples)
            }
        }
        _ => None,
    };
    let examples_stored = StoredValue::new(example_words);

    let kanji_for_animation: Option<String> = match &card {
        DomainCard::Kanji(_) => Some(card.question().text().to_string()),
        _ => None,
    };
    let kanji_stored = StoredValue::new(kanji_for_animation);

    let lesson_ctx = use_context::<LessonContext>();
    let question_text = question.get_value();

    let is_expanded = RwSignal::new(false);
    let content_ref = NodeRef::<leptos::html::Div>::new();
    let needs_collapse = RwSignal::new(false);

    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get())
            .unwrap_or(false);
        if !show_answer && card_type != CardType::Kanji && is_speech_supported() && !is_muted {
            let reading = get_reading_from_text(&question_text);
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
        <Card class=Signal::derive(|| "p-6 min-h-[300px] flex flex-col".to_string()) shadow=Signal::derive(|| true)>
            <LessonCardHeader
                card_type=card_type
                question_text=question.get_value()
            />

            <div class="flex-1 flex flex-col justify-center">
                <Show when=move || !show_answer>
                    <LessonCardQuestion
                        question_text=question.get_value()
                        kanji=kanji_stored.get_value()
                        is_reversed=is_reversed
                        on_show_answer=on_show_answer
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
                        is_reversed=is_reversed
                    />

                    <Show when=move || card_type == CardType::Kanji && is_expanded.get()>
                        {move || {
                            kanji_stored.get_value().map(|kanji| {
                                view! {
                                    <KanjiCardDetails
                                        kanji=kanji
                                        radicals=radicals_stored.get_value()
                                        example_words=examples_stored.get_value()
                                        show_details=is_expanded.get()
                                    />
                                }
                            })
                        }}
                    </Show>
                </Show>
            </div>
        </Card>
    }
}
