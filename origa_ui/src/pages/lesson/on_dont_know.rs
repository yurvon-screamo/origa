use super::lesson_state::LessonState;
use leptos::prelude::*;
use origa::domain::{LessonCardView, QuizMode, Rating};

pub fn create_on_dont_know(
    lesson_state: RwSignal<LessonState>,
    on_rate_callback: Callback<Rating>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let state = lesson_state.get();
        let is_phrase = state.current_index >= state.core_count;

        let is_multi_quiz = state
            .card_ids
            .get(state.current_index)
            .and_then(|id| state.cards.get(id))
            .map(|c| {
                matches!(
                    c.view(),
                    LessonCardView::KanjiReadingQuiz(q) if q.mode() == QuizMode::Multi
                )
            })
            .unwrap_or(false);

        lesson_state.update(|state| {
            state.dont_know_selected = true;
            state.selected_quiz_option = None;
            state.selected_yesno_answer = None;
            state.showing_answer = true;
            state.selected_quiz_options.clear();
            state.multi_result = None;
        });

        if is_phrase || is_multi_quiz {
            lesson_state.update(|state| {
                state.waiting_for_next = true;
                state.pending_rating = Some(Rating::Again);
            });
        } else {
            let on_rate_clone = on_rate_callback;
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(1500).await;
                on_rate_clone.run(Rating::Again);
            });
        }
    })
}
