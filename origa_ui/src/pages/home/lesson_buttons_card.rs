use crate::i18n::*;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

#[component]
pub fn LessonButtonsCard(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_lesson = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "lesson-buttons-lesson".to_string()
        } else {
            format!("{}-lesson", val)
        }
    });

    view! {
        <A href="/lesson" attr:data-testid=move || {
            let val = test_id.get();
            if val.is_empty() { None } else { Some(val) }
        }>
            <button
                class="btn btn-olive flex items-center justify-center gap-1.5 px-3 py-2 sm:px-4 sm:gap-2 card-shadow"
                data-testid=move || {
                    let val = test_id_lesson.get();
                    if val.is_empty() { None } else { Some(val) }
                }
            >
                <Icon icon=icondata::LuBookOpen width="16" height="16" />
                <span class="hidden sm:inline">{t!(i18n, home.lesson)}</span>
            </button>
        </A>
    }
}
