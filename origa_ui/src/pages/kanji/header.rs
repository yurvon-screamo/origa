use super::add_kanji_modal::AddKanjiModal;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use icondata::LuArrowLeft;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_navigate;

#[component]
pub fn KanjiHeader(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();
    let is_modal_open = RwSignal::new(false);

    view! {
        <div class="flex items-center gap-3 mb-6">
            <Button
                variant=ButtonVariant::Ghost
                test_id="kanji-back-btn"
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    navigate("/home", Default::default());
                })
            >
                <Icon icon=LuArrowLeft width="16" height="16" />
                {t!(i18n, common.back)}
            </Button>
            <Heading level=HeadingLevel::H1 test_id="kanji-title">
                {t!(i18n, kanji_page.header)}
            </Heading>
            <div class="ml-auto flex items-center gap-2 sm:gap-4">
                <Button
                    variant=ButtonVariant::Olive
                    test_id="kanji-add-btn"
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        is_modal_open.set(true);
                    })
                >
                    "+"
                </Button>
            </div>
        </div>

        <AddKanjiModal is_open=is_modal_open refresh_trigger=refresh_trigger />
    }
}
