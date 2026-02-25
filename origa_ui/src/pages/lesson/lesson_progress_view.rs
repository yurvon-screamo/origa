use super::lesson_progress::LessonProgress;
use super::lesson_state::LessonContext;
use leptos::prelude::*;

#[component]
pub fn LessonProgressView() -> impl IntoView {
    let lesson_state = use_context::<LessonContext>()
        .expect("lesson context")
        .lesson_state;

    let current = Signal::derive(move || lesson_state.get().current_index + 1);
    let total = Signal::derive(move || lesson_state.get().card_ids.len());

    view! {
        <LessonProgress current=current total=total />
    }
}
