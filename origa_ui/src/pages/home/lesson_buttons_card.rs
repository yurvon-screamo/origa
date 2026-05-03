use crate::i18n::*;
use leptos::prelude::*;
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
        <div data-testid=move || {
            let val = test_id.get();
            if val.is_empty() { None } else { Some(val) }
        }>
            <A href="/lesson">
                <button
                    class="btn btn-olive w-full h-16 py-0 flex items-center justify-center card-shadow"
                    data-testid=move || {
                        let val = test_id_lesson.get();
                        if val.is_empty() { None } else { Some(val) }
                    }
                >
                    {t!(i18n, home.lesson)}
                </button>
            </A>
        </div>
    }
}
