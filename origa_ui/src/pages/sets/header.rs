use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use icondata::LuArrowLeft;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_navigate;

#[component]
pub fn SetsHeader() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();

    view! {
        <div class="flex items-center gap-3 mb-6">
            <Button
                variant=ButtonVariant::Ghost
                test_id="sets-back-btn"
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    navigate("/words", Default::default());
                })
            >
                <Icon icon=LuArrowLeft width="16" height="16" />
                {t!(i18n, common.back)}
            </Button>
            <Heading level=HeadingLevel::H1 test_id="sets-title">
                {t!(i18n, sets.header)}
            </Heading>
        </div>
    }
}
