use super::lesson_state::LessonState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{LessonCardView, Rating};

pub fn create_on_quiz_submit(
    lesson_state: RwSignal<LessonState>,
    on_rate_callback: Callback<Rating>,
) -> Callback<()> {
    let Some(is_disposed) = use_context::<StoredValue<()>>() else {
        return Callback::new(move |_: ()| {});
    };

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

        let LessonCardView::KanjiReadingQuiz(quiz) = lesson_card.view() else {
            return;
        };

        let multi_result = quiz.check_multi_answers(&selected);
        let rating = multi_result.rating();

        lesson_state.update(|state| {
            state.showing_answer = true;
            state.multi_quiz_submitted = true;
            state.multi_result = Some(multi_result);
        });

        if is_phrase {
            lesson_state.update(|state| {
                state.waiting_for_next = true;
                state.pending_rating = Some(rating);
            });
        } else {
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
