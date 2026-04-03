use super::add_kanji_modal::AddKanjiModal;
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn KanjiHeader(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let navigate = use_navigate();
    let is_modal_open = RwSignal::new(false);

    let navigate_to_home = navigate.clone();

    view! {
        <div class="flex flex-wrap justify-between items-center gap-4 mb-6">
            <Heading level=HeadingLevel::H1 test_id="kanji-title">
                "Кандзи"
            </Heading>
            <div class="flex items-center gap-2 sm:gap-4">
                <Button
                    variant=ButtonVariant::Ghost
                    test_id="kanji-back-btn"
                    on_click=Callback::new(move |_| {
                        navigate_to_home("/home", Default::default());
                    })
                >
                    "Назад"
                </Button>
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
