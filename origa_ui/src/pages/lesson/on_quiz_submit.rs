use super::lesson_state::LessonState;
use leptos::prelude::*;
use origa::domain::{LessonCardView, QuizMode, Rating};

pub fn create_on_quiz_submit(
    lesson_state: RwSignal<LessonState>,
    on_rate_callback: Callback<Rating>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let state = lesson_state.get();
        let selected: Vec<usize> = state.selected_quiz_options.iter().copied().collect();

        if selected.is_empty() {
            return;
        }

        let is_phrase = state.current_index >= state.core_count;
        let card_id = state.card_ids.get(state.current_index).copied();

        let Some(card_id) = card_id else {
            return;
        };

        let Some(lesson_card) = state.cards.get(&card_id) else {
            return;
        };

        let quiz = match lesson_card.view() {
            LessonCardView::KanjiReadingQuiz(q) => q,
            _ => return,
        };

        let is_multi_quiz = quiz.mode() == QuizMode::Multi;
        let multi_result = quiz.check_multi_answers(&selected);
        let rating = multi_result.rating();

        lesson_state.update(|state| {
            state.showing_answer = true;
            state.multi_quiz_submitted = true;
            state.multi_result = Some(multi_result);
        });

        if is_phrase || is_multi_quiz {
            lesson_state.update(|state| {
                state.waiting_for_next = true;
                state.pending_rating = Some(rating);
            });
        } else {
            let on_rate_clone = on_rate_callback;
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(1500).await;
                on_rate_clone.run(rating);
            });
        }
    })
}
