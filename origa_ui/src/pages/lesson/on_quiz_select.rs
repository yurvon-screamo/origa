use super::lesson_state::LessonState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{LessonCardView, Rating};

pub fn create_on_quiz_select(
    lesson_state: RwSignal<LessonState>,
    on_rate_callback: Callback<Rating>,
) -> Callback<usize> {
    let Some(is_disposed) = use_context::<StoredValue<()>>() else {
        return Callback::new(move |_: usize| {});
    };

    Callback::new(move |option_index: usize| {
        let state = lesson_state.get();
        let is_phrase = state.current_index >= state.core_count;

        lesson_state.update(|state| {
            state.selected_quiz_option = Some(option_index);
            state.showing_answer = true;
        });

        let Some(&card_id) = lesson_state
            .get()
            .card_ids
            .get(lesson_state.get().current_index)
        else {
            return;
        };

        if let Some(lesson_card) = lesson_state.get().cards.get(&card_id) {
            let is_correct = match lesson_card.view() {
                LessonCardView::Quiz(q) | LessonCardView::KanjiReadingQuiz(q) => {
                    Some(q.check_answer(option_index))
                },
                LessonCardView::GrammarQuiz(gq) => Some(gq.quiz().check_answer(option_index)),
                LessonCardView::PhraseListen { options, .. } => {
                    options.get(option_index).map(|o| o.is_correct())
                },
                _ => None,
            };

            if let Some(is_correct) = is_correct {
                let rating = if is_correct {
                    Rating::Good
                } else {
                    Rating::Hard
                };

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
            }
        }
    })
}
