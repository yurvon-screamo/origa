use super::lesson_state::LessonState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::Rating;

pub fn create_on_dont_know(
    lesson_state: RwSignal<LessonState>,
    on_rate_callback: Callback<Rating>,
) -> Callback<()> {
    let is_disposed = use_context::<StoredValue<()>>().expect("is_disposed must be provided");

    Callback::new(move |_: ()| {
        let state = lesson_state.get();
        let is_phrase = state.current_index >= state.core_count;

        lesson_state.update(|state| {
            state.dont_know_selected = true;
            state.selected_quiz_option = None;
            state.selected_yesno_answer = None;
            state.showing_answer = true;
        });

        if is_phrase {
            lesson_state.update(|state| {
                state.waiting_for_next = true;
                state.pending_rating = Some(Rating::Again);
            });
        } else {
            let on_rate_clone = on_rate_callback;
            spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(1500).await;
                if is_disposed.is_disposed() {
                    return;
                }
                on_rate_clone.run(Rating::Again);
            });
        }
    })
}
