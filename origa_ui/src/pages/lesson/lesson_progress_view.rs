use super::lesson_progress::LessonProgress;
use super::lesson_state::LessonContext;
use leptos::prelude::*;

#[component]
pub fn LessonProgressView() -> impl IntoView {
    let ctx = use_context::<LessonContext>().expect("lesson context");
    let lesson_state = ctx.lesson_state;
    let core_count = ctx.core_count;

    let current = Signal::derive(move || lesson_state.get().current_index + 1);
    let total = Signal::derive(move || lesson_state.get().card_ids.len());
    let core_count_signal = Signal::derive(move || core_count.get());

    view! {
        <LessonProgress current=current total=total core_count=core_count_signal />
    }
}
