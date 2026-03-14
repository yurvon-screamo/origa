use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn ProfileHeader(username: Signal<String>) -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <div class="flex flex-wrap justify-between items-center gap-4 mb-6">
            <div class="flex flex-col items-center space-y-4 flex-1">
                <Heading level=HeadingLevel::H1>
                    {move || format!("Профиль {}", username.get())}
                </Heading>
            </div>
            <div class="flex items-center gap-2">
                <Button
                    variant=ButtonVariant::Ghost
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate("/home", Default::default());
                    })
                >
                    "Назад"
                </Button>
            </div>
        </div>
    }
}
