use super::add_words_preview_modal::AddWordsPreviewModal;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use icondata::LuArrowLeft;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_navigate;

#[component]
pub fn WordsHeader(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();
    let navigate_home = navigate.clone();
    let navigate_sets = navigate.clone();
    let is_modal_open = RwSignal::new(false);

    view! {
        <div class="flex items-center gap-3 mb-6">
            <Button
                variant=ButtonVariant::Ghost
                test_id="words-back-btn"
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    navigate_home("/home", Default::default());
                })
            >
                <Icon icon=LuArrowLeft width="16" height="16" />
                {t!(i18n, common.back)}
            </Button>
            <Heading level=HeadingLevel::H1 test_id="words-title">
                {t!(i18n, words.header)}
            </Heading>
            <div class="ml-auto flex items-center gap-2 sm:gap-4">
                <Button
                    variant=ButtonVariant::Ghost
                    test_id="words-sets-btn"
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate_sets("/sets", Default::default());
                    })
                >
                    {t!(i18n, words.decks)}
                </Button>
                <Button
                    variant=ButtonVariant::Olive
                    test_id="words-add-btn"
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        is_modal_open.set(true);
                    })
                >
                    "+"
                </Button>
            </div>
        </div>

        <AddWordsPreviewModal is_open=is_modal_open refresh_trigger=refresh_trigger />
    }
}
