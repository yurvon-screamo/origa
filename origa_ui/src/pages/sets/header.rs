use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn SetsHeader() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();

    view! {
        <div class="flex flex-wrap justify-between items-center gap-4 mb-6">
            <Heading level=HeadingLevel::H2 test_id="sets-title">
                {t!(i18n, sets.header)}
            </Heading>
            <Button
                variant=ButtonVariant::Ghost
                test_id="sets-back-btn"
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    navigate("/words", Default::default());
                })
            >
                {t!(i18n, common.back)}
            </Button>
        </div>
    }
}
