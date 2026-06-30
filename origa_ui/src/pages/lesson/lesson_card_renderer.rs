use super::lesson_card::LessonCard as LessonCardComponent;
use super::phrase_rating_buttons::PhraseRatingButtons;
use super::rating_buttons_view::RatingButtonsView;
use leptos::prelude::*;
use origa::domain::{Card, GrammarInfo, LessonCard, LessonCardView, NativeLanguage, Rating};
use std::collections::HashSet;
use ulid::Ulid;

struct LessonCardParams {
    card: Card,
    is_reversed: bool,
    grammar_info: Option<GrammarInfo>,
}

pub(in crate::pages::lesson) fn render_lesson_card(
    lesson_card: LessonCard,
    show_answer: Signal<bool>,
    on_show_answer: Callback<()>,
    on_rate_callback: Callback<Rating>,
    is_rating: RwSignal<Option<Ulid>>,
    known_kanji: RwSignal<HashSet<char>>,
    native_language: RwSignal<NativeLanguage>,
) -> impl IntoView {
    let params = match lesson_card.into_view() {
        LessonCardView::Normal(card) => {
            let grammar_info = match &card {
                Card::Grammar(grc) => {
                    let lang = native_language.get();
                    let title = grc
                        .title(&lang)
                        .ok()
                        .map(|q| q.text().to_string())
                        .unwrap_or_default();
                    Some(GrammarInfo::new(Some(*grc.rule_id()), title, String::new()))
                },
                _ => None,
            };
            LessonCardParams {
                card,
                is_reversed: false,
                grammar_info,
            }
        },
        LessonCardView::Reversed(card) => LessonCardParams {
            card,
            is_reversed: true,
            grammar_info: None,
        },
        LessonCardView::GrammarMutated { card, grammar_info } => LessonCardParams {
            card,
            is_reversed: false,
            grammar_info: Some(grammar_info),
        },
        LessonCardView::Quiz(_)
        | LessonCardView::Writing(_)
        | LessonCardView::YesNo(_)
        | LessonCardView::PhraseListen { .. }
        | LessonCardView::KanjiReadingQuiz(_)
        | LessonCardView::GrammarQuiz(_) => {
            return ().into_any();
        },
    };

    let is_phrase = matches!(params.card, Card::Phrase(_));

    if is_phrase {
        let phrase_audio_path = match &params.card {
            Card::Phrase(pc) => Some(format!("phrases/audio/{}.opus", pc.phrase_id())),
            _ => None,
        };

        view! {
            <LessonCardComponent
                card=params.card
                is_reversed=params.is_reversed
                show_answer
                on_show_answer=on_show_answer
                grammar_info=params.grammar_info
                native_language=native_language.get()
                known_kanji=Signal::from(known_kanji)
                audio_path=phrase_audio_path
            />

            <Show when=move || show_answer.get()>
                <PhraseRatingButtons
                    on_rate=on_rate_callback
                    disabled=Signal::derive(move || is_rating.get().is_some())
                    test_id=Signal::derive(|| "lesson-phrase-rating".to_string())
                />
            </Show>
        }
        .into_any()
    } else {
        view! {
            <LessonCardComponent
                card=params.card
                is_reversed=params.is_reversed
                show_answer
                on_show_answer=on_show_answer
                grammar_info=params.grammar_info
                native_language=native_language.get()
                known_kanji=Signal::from(known_kanji)
                audio_path=None
            />

            <Show when=move || show_answer.get()>
                <RatingButtonsView
                    on_rate=on_rate_callback
                    disabled=Signal::derive(move || is_rating.get().is_some())
                />
            </Show>
        }
        .into_any()
    }
}
