use super::lesson_state::LessonState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{LessonCardView, Rating};

pub fn create_on_yesno_select(
    lesson_state: RwSignal<LessonState>,
    on_rate_callback: Callback<Rating>,
) -> Callback<bool> {
    let is_disposed = use_context::<StoredValue<()>>().expect("is_disposed must be provided");

    Callback::new(move |answer: bool| {
        lesson_state.update(|state| {
            state.selected_yesno_answer = Some(answer);
            state.showing_answer = true;
        });

        let state = lesson_state.get();
        let card_id = state.card_ids.get(state.current_index);

        if let Some(card_id) = card_id
            && let Some(card_view) = state.cards.get(card_id)
            && let LessonCardView::YesNo(yesno) = card_view
        {
            let is_correct = yesno.check_answer(answer);
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
