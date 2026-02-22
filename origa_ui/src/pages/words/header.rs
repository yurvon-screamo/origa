use super::add_word_modal::AddWordModal;
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn WordsHeader() -> impl IntoView {
    let navigate = use_navigate();
    let is_modal_open = RwSignal::new(false);

    view! {
        <div class="flex justify-between items-center mb-6">
            <Heading level=HeadingLevel::H1>
                "Слова"
            </Heading>
            <div class="flex items-center gap-4">
                <Button
                    variant=ButtonVariant::Ghost
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate("/home", Default::default());
                    })
                >
                    "Назад"
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

        <AddWordModal is_open=is_modal_open />
    }
}
