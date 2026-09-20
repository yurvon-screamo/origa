use super::lesson_card::LessonCard as LessonCardComponent;
use super::phrase_rating_buttons::PhraseRatingButtons;
use super::rating_buttons_view::RatingButtonsView;
use leptos::prelude::*;
use origa::domain::{Card, GrammarInfo, LessonCard, LessonCardView, NativeLanguage, Rating};
use std::collections::HashSet;

struct LessonCardParams {
    card: Card,
    is_reversed: bool,
    grammar_info: Option<GrammarInfo>,
    /// Grammar badge in the tags row — false for mutated word cards: the
    /// mutation rule already leads the answer body, a tag duplicate is
    /// noise (owner request).
    show_grammar_badge: bool,
}

pub(in crate::pages::lesson) fn render_lesson_card(
    lesson_card: LessonCard,
    show_answer: Signal<bool>,
    on_show_answer: Callback<()>,
    on_rate_callback: Callback<Rating>,
    disabled: Signal<bool>,
    known_kanji: Signal<HashSet<char>>,
    native_language: RwSignal<NativeLanguage>,
) -> impl IntoView {
    let params = match lesson_card.into_view() {
        LessonCardView::Normal(card) => {
            let grammar_info = match &card {
                Card::Grammar(grc) => {
                    let lang = native_language.get();
                    let title = grc
                        .pattern(&lang)
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
                show_grammar_badge: true,
            }
        },
        LessonCardView::Reversed(card) => LessonCardParams {
            card,
            is_reversed: true,
            grammar_info: None,
            show_grammar_badge: true,
        },
        // AudioRecall degrades to the Normal rendering path when audio is
        // unavailable (the container decides via `audio_mode_active`); the
        // renderer itself is degradation-agnostic and maps the variant
        // unconditionally.
        LessonCardView::AudioRecall(card) => LessonCardParams {
            card,
            is_reversed: false,
            grammar_info: None,
            show_grammar_badge: true,
        },
        // Muted PhraseListen degrades the same way (the container gates it
        // via `audio_mode_active`): the base phrase card renders as Normal
        // — translations + rating buttons. The quiz fields (audio_file,
        // options) are dropped; the Normal path builds its own phrase
        // audio path from the phrase id.
        LessonCardView::PhraseListen { card, .. } => LessonCardParams {
            card,
            is_reversed: false,
            grammar_info: None,
            show_grammar_badge: true,
        },
        LessonCardView::GrammarMutated { card, grammar_info } => LessonCardParams {
            card,
            is_reversed: false,
            grammar_info: Some(grammar_info),
            show_grammar_badge: false,
        },
        LessonCardView::Quiz(_)
        | LessonCardView::Writing(_)
        | LessonCardView::YesNo(_)
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
                known_kanji=known_kanji
                audio_path=phrase_audio_path
                show_grammar_badge=params.show_grammar_badge
            />

            <Show when=move || show_answer.get()>
                <PhraseRatingButtons
                    on_rate=on_rate_callback
                    disabled=disabled
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
                known_kanji=known_kanji
                audio_path=None
                show_grammar_badge=params.show_grammar_badge
            />

            <Show when=move || show_answer.get()>
                <RatingButtonsView
                    on_rate=on_rate_callback
                    disabled=disabled
                />
            </Show>
        }
        .into_any()
    }
}
