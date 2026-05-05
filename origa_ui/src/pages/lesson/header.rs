use super::lesson_state::LessonContext;
use crate::i18n::use_i18n;
use crate::ui_components::PageHeader;
use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn LessonHeader() -> impl IntoView {
    let i18n = use_i18n();
    let lesson_ctx = use_context::<LessonContext>().expect("LessonContext not provided");
    let is_muted = lesson_ctx.is_muted;

    let toggle_mute = move || {
        is_muted.update(|m| *m = !*m);
    };

    view! {
        <PageHeader
            back_path="/home".to_string()
            back_label=Signal::derive(move || i18n.get_keys().common().back().inner().to_string())
            test_id="lesson"
        >
            <button
                data-testid="lesson-mute-btn"
                class="btn btn-ghost px-3 py-2"
                data-muted=move || if is_muted.get() { "true" } else { "false" }
                on:click=move |_| toggle_mute()
            >
                {move || if is_muted.get() {
                    view! { <Icon icon=icondata::LuVolumeX width="1.25em" height="1.25em" /> }
                        .into_any()
                } else {
                    view! { <Icon icon=icondata::LuVolume2 width="1.25em" height="1.25em" /> }
                        .into_any()
                }}
            </button>
        </PageHeader>
    }
}
