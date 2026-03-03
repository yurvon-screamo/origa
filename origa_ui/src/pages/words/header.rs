use super::add_words_preview_modal::AddWordsPreviewModal;
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn WordsHeader() -> impl IntoView {
    let navigate = use_navigate();
    let navigate_clone = navigate.clone();
    let is_modal_open = RwSignal::new(false);

    view! {
        <div class="flex flex-wrap justify-between items-center gap-4 mb-6">
            <Heading level=HeadingLevel::H1>
                "Слова"
            </Heading>
            <div class="flex items-center gap-2 sm:gap-4">
                <Button
                    variant=ButtonVariant::Ghost
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate("/home", Default::default());
                    })
                >
                    "Назад"
                </Button>
                <Button
                    variant=ButtonVariant::Ghost
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate_clone("/sets", Default::default());
                    })
                >
                    "Колоды"
                </Button>
                <Button
                    variant=ButtonVariant::Olive
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        is_modal_open.set(true);
                    })
                >
                    "+"
                </Button>
            </div>
        </div>

        <AddWordsPreviewModal is_open=is_modal_open />
    }
}
