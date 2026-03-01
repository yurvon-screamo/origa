use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn SetsHeader() -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <div class="flex flex-wrap justify-between items-center gap-4 mb-6">
            <Heading level=HeadingLevel::H2>
                "Наборы для изучения"
            </Heading>
            <Button
                variant=ButtonVariant::Ghost
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    navigate("/home", Default::default());
                })
            >
                "Назад"
            </Button>
        </div>
    }
}
