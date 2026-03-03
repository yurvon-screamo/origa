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
    lesson_state: RwSignal<LessonState>,
    show_answer: impl Fn() + 'static,
) -> impl Fn(KeyboardEvent) {
    move |ev: KeyboardEvent| {
        let key = ev.key();
        let state = lesson_state.get();

        if lesson_ctx.is_completed.get() || is_rating.get().is_some() {
            return;
        }

        let is_quiz = matches!(
            state
                .cards
                .get(state.card_ids.get(state.current_index).unwrap()),
            Some(LessonCardView::Quiz(_))
        );

        if is_quiz && !state.showing_answer {
            match key.as_str() {
                "1" => {
                    ev.prevent_default();
                    on_quiz_select.run(0);
                }
                "2" => {
                    ev.prevent_default();
                    on_quiz_select.run(1);
                }
                "3" => {
                    ev.prevent_default();
                    on_quiz_select.run(2);
                }
                "4" => {
                    ev.prevent_default();
                    on_quiz_select.run(3);
                }
                _ => {}
            }
            return;
        }

        match key.as_str() {
            " " if !state.showing_answer => {
                ev.prevent_default();
                show_answer();
            }
            "1" if state.showing_answer => {
                on_rate_callback.run(Rating::Again);
            }
            "2" if state.showing_answer => {
                on_rate_callback.run(Rating::Hard);
            }
            "3" if state.showing_answer => {
                on_rate_callback.run(Rating::Good);
            }
            "4" if state.showing_answer => {
                on_rate_callback.run(Rating::Easy);
            }
            _ => {}
        }
    }
}
