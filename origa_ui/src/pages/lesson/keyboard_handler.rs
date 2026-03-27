use super::lesson_state::LessonContext;
use super::lesson_state::LessonState;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use origa::domain::{LessonCardView, Rating};
use ulid::Ulid;

pub fn create_keyboard_handler(
    lesson_ctx: LessonContext,
    is_rating: RwSignal<Option<Ulid>>,
    on_rate_callback: Callback<Rating>,
    on_quiz_select: Callback<usize>,
    on_yesno_select: Callback<bool>,
    lesson_state: RwSignal<LessonState>,
    show_answer: impl Fn() + 'static,
) -> impl Fn(KeyboardEvent) {
    move |ev: KeyboardEvent| {
        let key = ev.key();
        let state = lesson_state.get();

        if lesson_ctx.is_completed.get() || is_rating.get().is_some() {
            return;
        }

        let current_card_id = state.card_ids.get(state.current_index);
        let current_card = current_card_id.and_then(|id| state.cards.get(id));

        let is_quiz = matches!(current_card, Some(LessonCardView::Quiz(_)));
        let is_yesno = matches!(current_card, Some(LessonCardView::YesNo(_)));

        if !state.showing_answer {
            if is_quiz {
                match key.as_str() {
                    "1" => {
                        ev.prevent_default();
                        on_quiz_select.run(0);
                    },
                    "2" => {
                        ev.prevent_default();
                        on_quiz_select.run(1);
                    },
                    "3" => {
                        ev.prevent_default();
                        on_quiz_select.run(2);
                    },
                    "4" => {
                        ev.prevent_default();
                        on_quiz_select.run(3);
                    },
                    _ => {},
                }
                return;
            }

            if is_yesno {
                match key.as_str() {
                    "1" => {
                        ev.prevent_default();
                        on_yesno_select.run(false);
                    },
                    "2" => {
                        ev.prevent_default();
                        on_yesno_select.run(true);
                    },
                    _ => {},
                }
                return;
            }
        }

        if state.showing_answer && !is_quiz && !is_yesno {
            match key.as_str() {
                "1" => {
                    on_rate_callback.run(Rating::Again);
                },
                "2" => {
                    on_rate_callback.run(Rating::Hard);
                },
                "3" => {
                    on_rate_callback.run(Rating::Good);
                },
                "4" => {
                    on_rate_callback.run(Rating::Easy);
                },
                _ => {},
            }
            return;
        }

        if key == " " && !state.showing_answer {
            ev.prevent_default();
            show_answer();
        }
    }
}
