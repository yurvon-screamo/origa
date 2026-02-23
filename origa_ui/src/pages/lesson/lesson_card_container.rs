use super::lesson_card::LessonCard;
use super::lesson_state::LessonContext;
use super::rating_buttons_view::RatingButtonsView;
use leptos::prelude::*;

#[component]
pub fn LessonCardContainer() -> impl IntoView {
    let lesson_ctx = use_context::<LessonContext>().expect("lesson context");
    let lesson_state = lesson_ctx.lesson_state;

    let show_answer = move || {
        lesson_state.update(|state| {
            state.showing_answer = true;
        });
    };

    let current_card = Memo::new(move |_| {
        let state = lesson_state.get();
        state
            .card_ids
            .get(state.current_index)
            .and_then(|id| state.cards.get(id))
            .cloned()
    });

    view! {
        <Show when=move || current_card.get().is_some()>
            <LessonCard
                card=current_card.get().unwrap()
                show_answer=lesson_state.get().showing_answer
                on_show_answer=Callback::new(move |_| show_answer())
            />

            <Show when=move || lesson_state.get().showing_answer>
                <RatingButtonsView />
            </Show>
        </Show>
    }
}
