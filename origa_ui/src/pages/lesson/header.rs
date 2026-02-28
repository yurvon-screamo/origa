use super::lesson_state::LessonContext;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn LessonHeader() -> impl IntoView {
    let lesson_ctx = use_context::<LessonContext>().expect("LessonContext not provided");
    let is_muted = lesson_ctx.is_muted;

    let toggle_mute = move || {
        is_muted.update(|m| *m = !*m);
    };

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
            <button
                class="btn btn-ghost btn-sm px-3 py-2 text-lg"
                on:click=move |_| toggle_mute()
            >
                {move || if is_muted.get() { "🔇" } else { "🔊" }}
            </button>
        </div>
    }
}
