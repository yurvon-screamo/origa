use super::lesson_state::LessonContext;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn LessonHeader() -> impl IntoView {
    let lesson_ctx = use_context::<LessonContext>().expect("LessonContext not provided");

    view! {
        <div class="flex justify-between items-center mb-6">
            <A href="/home">
                <Button variant=Signal::derive(|| ButtonVariant::Ghost)>
                    "Назад"
                </Button>
            </A>
            <h1 class="font-serif text-2xl font-light tracking-tight">
                {lesson_ctx.mode.title()}
            </h1>
            <div class="w-16"></div>
        </div>
    }
}
