use super::lesson_progress::LessonProgress;
use super::lesson_state::LessonContext;
use leptos::prelude::*;

#[component]
pub fn LessonProgressView() -> impl IntoView {
    let lesson_state = use_context::<LessonContext>()
        .expect("lesson context")
        .lesson_state;
    let state = lesson_state.get();
    let total = state.card_ids.len();
    let current = state.current_index + 1;

    view! {
        <LessonProgress current=current total=total />
    }
}
