use super::lesson_state::LessonContext;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

#[component]
pub fn LessonHeader() -> impl IntoView {
    let lesson_ctx = use_context::<LessonContext>().expect("LessonContext not provided");
    let is_muted = lesson_ctx.is_muted;

    let toggle_mute = move || {
        is_muted.update(|m| *m = !*m);
    };

    view! {
        <div class="flex flex-wrap justify-between items-center gap-4 mb-6">
            <A href="/home">
                <Button variant=Signal::derive(|| ButtonVariant::Ghost)>
                    "Назад"
                </Button>
            </A>
            <h1 class="font-serif text-2xl font-light tracking-tight">
                {lesson_ctx.mode.title()}
            </h1>
            <button
                class="btn btn-ghost px-3 py-2"
                on:click=move |_| toggle_mute()
            >
                {move || if is_muted.get() {
                    view! { <Icon icon=icondata::LuVolumeX width="1.25em" height="1.25em" /> }.into_any()
                } else {
                    view! { <Icon icon=icondata::LuVolume2 width="1.25em" height="1.25em" /> }.into_any()
                }}
            </button>
        </div>
    }
}
