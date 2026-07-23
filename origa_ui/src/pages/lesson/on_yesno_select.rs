use super::lesson_state::LessonState;
use leptos::prelude::*;
use origa::domain::{LessonCardView, Rating};

pub fn create_on_yesno_select(lesson_state: RwSignal<LessonState>) -> Callback<bool> {
    // Defensive: without a dispose sentinel in context the handler returns a
    // no-op callback. Same pattern as on_quiz_select — see its doc comment.
    if use_context::<StoredValue<()>>().is_none() {
        return Callback::new(move |_: bool| {});
    }

    Callback::new(move |answer: bool| {
        lesson_state.update(|state| {
            state.selected_yesno_answer = Some(answer);
            state.showing_answer = true;
        });

        let state = lesson_state.get();
        let Some(card_id) = state.card_ids.get(state.current_index) else {
            return;
        };
        let Some(lesson_card) = state.cards.get(card_id) else {
            return;
        };
        let LessonCardView::YesNo(yesno) = lesson_card.view() else {
            return;
        };

        let rating = if yesno.check_answer(answer) {
            Rating::Good
        } else {
            Rating::Again
        };

        // Pure-manual advance (ADR-033): the user dismisses the feedback card
        // themselves via Space/Enter/click. Replaces the previous 1500ms timer.
        lesson_state.update(|state| {
            state.waiting_for_next = true;
            state.pending_rating = Some(rating);
        });
    })
}
