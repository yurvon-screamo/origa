use super::lesson_state::LessonState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{LessonCardView, Rating};

pub fn create_on_quiz_select(
    lesson_state: RwSignal<LessonState>,
    on_rate_callback: Callback<Rating>,
) -> Callback<usize> {
    let is_disposed = use_context::<StoredValue<()>>().expect("is_disposed must be provided");

    Callback::new(move |option_index: usize| {
        lesson_state.update(|state| {
            state.selected_quiz_option = Some(option_index);
            state.showing_answer = true;
        });

        if let Some(lesson_card) = lesson_state.get().cards.get(
            lesson_state
                .get()
                .card_ids
                .get(lesson_state.get().current_index)
                .unwrap(),
        ) && let LessonCardView::Quiz(quiz) = lesson_card.view()
        {
            let is_correct = quiz.check_answer(option_index);
            let rating = if is_correct {
                Rating::Good
            } else {
                Rating::Hard
            };

            let on_rate_clone = on_rate_callback;
            spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(1500).await;
                if is_disposed.is_disposed() {
                    return;
                }
                on_rate_clone.run(rating);
            });
        }
    })
}

pub fn create_on_quiz_dont_know(
    lesson_state: RwSignal<LessonState>,
    on_rate_callback: Callback<Rating>,
) -> Callback<()> {
    let is_disposed = use_context::<StoredValue<()>>().expect("is_disposed must be provided");

    Callback::new(move |_: ()| {
        lesson_state.update(|state| {
            state.dont_know_selected = true;
            state.selected_quiz_option = None;
            state.showing_answer = true;
        });

        let on_rate_clone = on_rate_callback;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(1500).await;
            if is_disposed.is_disposed() {
                return;
            }
            on_rate_clone.run(Rating::Again);
        });
    })
}
