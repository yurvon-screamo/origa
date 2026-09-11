use super::audio_recall_card::AudioRecallCardView;
use super::keyboard_handler::{KeyboardActions, create_keyboard_handler, is_typing_target};
use super::lesson_card_renderer::render_lesson_card;
use super::lesson_state::LessonContext;
use super::on_audio_select::create_on_audio_select;
use super::on_dont_know::create_on_dont_know;
use super::on_quiz_select::create_on_quiz_select;
use super::on_quiz_submit::create_on_quiz_submit;
use super::on_quiz_toggle::create_on_quiz_toggle;
use super::on_rate::create_on_rate_callback;
use super::on_yesno_select::create_on_yesno_select;
use super::phrase_card::PhraseCardView;
use super::quiz_card::QuizCardView;
use super::quiz_card::QuizVariant;
use super::writing_card::WritingCard;
use super::yesno_card_view::YesNoCardView;
use crate::pages::lesson::card_type::CardType;
use crate::ui_components::stop_current_audio;
use leptos::prelude::*;
use leptos_use::use_event_listener;
use origa::domain::{CardAnswer, LessonCardView, Rating};
use ulid::Ulid;

#[component]
pub fn LessonCardContainer() -> impl IntoView {
    let lesson_ctx = use_context::<LessonContext>().expect("lesson context");
    let lesson_state = lesson_ctx.lesson_state;
    let audio_mode_active = lesson_ctx.audio_mode_active;
    let is_rating = RwSignal::new(None::<Ulid>);
    let known_kanji = lesson_ctx.known_kanji;
    let native_language = lesson_ctx.native_language;

    let show_answer = move || {
        lesson_state.update(|state| {
            state.showing_answer = true;
        });
    };

    let on_rate_callback = create_on_rate_callback(lesson_state, lesson_ctx.clone(), is_rating);

    let on_quiz_select = create_on_quiz_select(lesson_state);

    let on_yesno_select = create_on_yesno_select(lesson_state);

    let on_quiz_toggle = create_on_quiz_toggle(lesson_state);
    let on_quiz_submit = create_on_quiz_submit(lesson_state);

    let on_quiz_dont_know = create_on_dont_know(lesson_state);
    let on_yesno_dont_know = create_on_dont_know(lesson_state);

    let on_audio_select = create_on_audio_select(lesson_state);

    // Replay speaks the CURRENT card's word: manual replay is an explicit
    // user action and is not gated by the lesson mute (a textless card
    // without sound is unanswerable).
    let on_replay_audio = Callback::new(move |_: ()| {
        let state = lesson_state.get_untracked();
        let Some(slot_id) = state.card_ids.get(state.current_index) else {
            return;
        };
        let Some(lesson_card) = state.cards.get(slot_id) else {
            return;
        };
        if !matches!(lesson_card.view(), LessonCardView::AudioRecall(_)) {
            return;
        }
        let word = lesson_card
            .card()
            .question(&native_language.get_untracked())
            .map(|q| q.text().to_string())
            .unwrap_or_default();
        crate::ui_components::speak_word(&word, 1.0);
    });

    let on_next_card = Callback::new(move |_: ()| {
        // Pure-manual advance (ADR-033) contract: every on_* handler that
        // flips `waiting_for_next` MUST also set `pending_rating`. A `None`
        // here means a handler forgot — fail loudly in debug. In release we
        // fall back to `Rating::Again`, the more conservative SRS choice
        // (re-show the card soon) rather than `Good`, which would silently
        // inflate intervals and skew FSRS parameters.
        let state = lesson_state.get();
        debug_assert!(
            state.pending_rating.is_some(),
            "on_next_card invoked without pending_rating — handler bug"
        );
        let rating = state.pending_rating.unwrap_or(Rating::Again);
        on_rate_callback.run(rating);
    });

    let handle_keydown = create_keyboard_handler(
        lesson_ctx,
        is_rating,
        lesson_state,
        KeyboardActions {
            on_rate: on_rate_callback,
            on_quiz_select,
            on_yesno_select,
            on_quiz_dont_know,
            on_yesno_dont_know,
            on_quiz_toggle,
            on_quiz_submit,
            on_audio_answer: on_audio_select,
            on_replay_audio,
            show_answer: Box::new(show_answer),
            on_next_card,
        },
    );

    let current_lesson_card = Memo::new(move |_| {
        let state = lesson_state.get();
        state
            .card_ids
            .get(state.current_index)
            .and_then(|id| state.cards.get(id))
            .cloned()
    });

    let is_quiz_mode = Memo::new(move |_| {
        current_lesson_card
            .get()
            .map(|c| matches!(c.view(), LessonCardView::Quiz(_)))
            .unwrap_or(false)
    });

    let is_yesno_mode = Memo::new(move |_| {
        current_lesson_card
            .get()
            .map(|c| matches!(c.view(), LessonCardView::YesNo(_)))
            .unwrap_or(false)
    });

    let is_writing_mode = Memo::new(move |_| {
        current_lesson_card
            .get()
            .map(|c| matches!(c.view(), LessonCardView::Writing(_)))
            .unwrap_or(false)
    });

    let is_phrase_listen_mode = Memo::new(move |_| {
        current_lesson_card
            .get()
            .map(|c| matches!(c.view(), LessonCardView::PhraseListen { .. }))
            .unwrap_or(false)
    });

    let is_kanji_reading_quiz_mode = Memo::new(move |_| {
        current_lesson_card
            .get()
            .map(|c| matches!(c.view(), LessonCardView::KanjiReadingQuiz(_)))
            .unwrap_or(false)
    });

    let is_grammar_quiz_mode = Memo::new(move |_| {
        current_lesson_card
            .get()
            .map(|c| matches!(c.view(), LessonCardView::GrammarQuiz(_)))
            .unwrap_or(false)
    });

    // Live AudioRecall showing: the view says AudioRecall AND the mode was
    // sampled available for this showing (audio source + not muted). A
    // degraded AudioRecall card falls through to the default branch below,
    // where the renderer maps it to the Normal path — one routing scheme,
    // no empty render in any combination.
    let is_audio_recall_active = Memo::new(move |_| {
        current_lesson_card
            .get()
            .map(|c| matches!(c.view(), LessonCardView::AudioRecall(_)))
            .unwrap_or(false)
            && audio_mode_active.get()
    });

    on_cleanup(move || {
        stop_current_audio();
    });

    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if is_typing_target(ev.target().as_ref()) {
            return;
        }
        handle_keydown(ev);
    });

    view! {
        <Show when=move || current_lesson_card.get().is_some()>
            <Show when=move || !is_quiz_mode.get() && !is_writing_mode.get() && !is_yesno_mode.get() && !is_phrase_listen_mode.get() && !is_kanji_reading_quiz_mode.get() && !is_grammar_quiz_mode.get() && !is_audio_recall_active.get()>
                {move || {
                    current_lesson_card.get().map(|lesson_card| {
                        render_lesson_card(
                            lesson_card,
                            Signal::derive(move || lesson_state.get().showing_answer),
                            Callback::new(move |_| show_answer()),
                            on_rate_callback,
                            is_rating,
                            known_kanji,
                            native_language,
                        )
                    })
                }}
            </Show>

            <Show when=move || is_quiz_mode.get()>
                {move || {
                    current_lesson_card.get().and_then(|lesson_card| {
                        if let LessonCardView::Quiz(quiz) = lesson_card.into_view() {
                            let state = lesson_state.get();
                            let selected_option = state.selected_quiz_option;

                            Some(view! {
                                <QuizCardView
                                    quiz_card=quiz
                                    show_result=Signal::derive(move || lesson_state.get().showing_answer)
                                    selected_option=selected_option
                                    on_select_option=on_quiz_select
                                    on_dont_know=on_quiz_dont_know
                                    dont_know_selected=Signal::derive(move || lesson_state.get().dont_know_selected)
                                    native_language=native_language.get()
                                    known_kanji=Signal::from(known_kanji)
                                    waiting_for_next=Signal::derive(move || lesson_state.get().waiting_for_next)
                                    on_next_card=on_next_card
                                />
                            })
                        } else {
                            None
                        }
                    })
                }}
            </Show>

            <Show when=move || is_writing_mode.get()>
                {move || {
                    current_lesson_card.get().and_then(|lesson_card| {
                        if let LessonCardView::Writing(card) = lesson_card.into_view() {
                            Some(view! {
                                <WritingCard
                                    card=card
                                    on_rate=on_rate_callback
                                    on_show_answer=Callback::new(move |_| show_answer())
                                    disabled=Signal::derive(move || is_rating.get().is_some())
                                    native_language=native_language.get()
                                    known_kanji=Signal::from(known_kanji)
                                />
                            })
                        } else {
                            None
                        }
                    })
                }}
            </Show>

            <Show when=move || is_yesno_mode.get()>
                {move || {
                    current_lesson_card.get().and_then(|lesson_card| {
                        if let LessonCardView::YesNo(yesno) = lesson_card.into_view() {
                            let state = lesson_state.get();
                            let selected_answer = state.selected_yesno_answer;

                            Some(view! {
                                <YesNoCardView
                                    yesno_card=yesno
                                    show_result=Signal::derive(move || lesson_state.get().showing_answer)
                                    selected_answer=selected_answer
                                    on_answer=on_yesno_select
                                    on_dont_know=on_yesno_dont_know
                                    dont_know_selected=Signal::derive(move || lesson_state.get().dont_know_selected)
                                    native_language=native_language.get()
                                    known_kanji=Signal::from(known_kanji)
                                    waiting_for_next=Signal::derive(move || lesson_state.get().waiting_for_next)
                                    on_next_card=on_next_card
                                />
                            })
                        } else {
                            None
                        }
                    })
                }}
            </Show>

            <Show when=move || is_phrase_listen_mode.get()>
                {move || {
                    current_lesson_card.get().and_then(|lesson_card| {
                        if let LessonCardView::PhraseListen { card, audio_file, options } = lesson_card.into_view() {
                            let state = lesson_state.get();
                            let selected_option = state.selected_quiz_option;
                            let card_type = CardType::from(&card);
                            let phrase_text = card.question(&native_language.get()).ok().map(|q| q.text().to_string());
                            let phrase_translation = {
                                let lang = native_language.get();
                                match card.answer(&lang).ok() {
                                    Some(CardAnswer::Vocabulary {
                                        translations,
                                        description,
                                    }) => Some(crate::utils::text_format::format_vocabulary_answer(
                                        &translations,
                                        &description,
                                    )),
                                    Some(CardAnswer::Text(s)) => {
                                        Some(crate::utils::text_format::split_sentences_to_markdown(&s))
                                    },
                                    Some(CardAnswer::GrammarNuances { .. }) => None,
                                    None => Some(String::new()),
                                }
                            };

                            Some(view! {
                                <PhraseCardView
                                    card_type=card_type
                                    audio_file=audio_file
                                    options=options
                                    show_result=Signal::derive(move || lesson_state.get().showing_answer)
                                    selected_option=selected_option
                                    on_select_option=on_quiz_select
                                    on_dont_know=on_quiz_dont_know
                                    dont_know_selected=state.dont_know_selected
                                    phrase_text=phrase_text
                                    phrase_translation=phrase_translation
                                    known_kanji=Signal::from(known_kanji)
                                    waiting_for_next=Signal::derive(move || lesson_state.get().waiting_for_next)
                                    on_next_card=on_next_card
                                />
                            })
                        } else {
                            None
                        }
                    })
                }}
            </Show>

            <Show when=move || is_audio_recall_active.get()>
                {move || {
                    current_lesson_card.get().and_then(|lesson_card| {
                        if let LessonCardView::AudioRecall(card) = lesson_card.into_view() {
                            Some(view! {
                                <AudioRecallCardView
                                    card=card
                                    show_result=Signal::derive(move || lesson_state.get().showing_answer)
                                    on_answer=on_audio_select
                                    on_reveal=Callback::new(move |_| show_answer())
                                    on_replay=on_replay_audio
                                    native_language=native_language.get()
                                    known_kanji=Signal::from(known_kanji)
                                    waiting_for_next=Signal::derive(move || lesson_state.get().waiting_for_next)
                                    on_next_card=on_next_card
                                />
                            })
                        } else {
                            None
                        }
                    })
                }}
            </Show>

            <Show when=move || is_kanji_reading_quiz_mode.get()>
                {move || {
                    current_lesson_card.get().and_then(|lesson_card| {
                        if let LessonCardView::KanjiReadingQuiz(quiz) = lesson_card.into_view() {
                            let state = lesson_state.get();
                            let selected_option = state.selected_quiz_option;
                            let selected_options = state.selected_quiz_options.clone();
                            let multi_result = state.multi_result;

                            Some(view! {
                                <QuizCardView
                                    quiz_card=quiz
                                    show_result=Signal::derive(move || lesson_state.get().showing_answer)
                                    selected_option=selected_option
                                    on_select_option=on_quiz_select
                                    on_dont_know=on_quiz_dont_know
                                    dont_know_selected=Signal::derive(move || lesson_state.get().dont_know_selected)
                                    native_language=native_language.get()
                                    known_kanji=Signal::from(known_kanji)
                                    quiz_variant=QuizVariant::Reading
                                    selected_options=Signal::derive(move || selected_options.clone())
                                    multi_submitted=Signal::derive(move || lesson_state.get().multi_quiz_submitted)
                                    multi_result=multi_result
                                    on_toggle=on_quiz_toggle
                                    on_submit=on_quiz_submit
                                    waiting_for_next=Signal::derive(move || lesson_state.get().waiting_for_next)
                                    on_next_card=on_next_card
                                    lenient_grading=true
                                />
                            })
                        } else {
                            None
                        }
                    })
                }}
            </Show>

            <Show when=move || is_grammar_quiz_mode.get()>
                {move || {
                    current_lesson_card.get().and_then(|lesson_card| {
                        if let LessonCardView::GrammarQuiz(gq) = lesson_card.into_view() {
                            let state = lesson_state.get();
                            let selected_option = state.selected_quiz_option;
                            // QuizCardView receives only the inner QuizCard (rule title
                            // as question, inflected forms as options). Forward the
                            // base word explicitly so it stays visible (#502).
                            let base_word = gq.word_text().to_string();

                            Some(view! {
                                <QuizCardView
                                    quiz_card=gq.quiz().clone()
                                    show_result=Signal::derive(move || lesson_state.get().showing_answer)
                                    selected_option=selected_option
                                    on_select_option=on_quiz_select
                                    on_dont_know=on_quiz_dont_know
                                    dont_know_selected=Signal::derive(move || lesson_state.get().dont_know_selected)
                                    native_language=native_language.get()
                                    known_kanji=Signal::from(known_kanji)
                                    quiz_variant=QuizVariant::Grammar
                                    grammar_base_word=Some(base_word)
                                    waiting_for_next=Signal::derive(move || lesson_state.get().waiting_for_next)
                                    on_next_card=on_next_card
                                />
                            })
                        } else {
                            None
                        }
                    })
                }}
            </Show>
        </Show>
    }
}
