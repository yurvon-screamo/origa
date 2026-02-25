use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use origa::domain::User;

#[component]
pub fn ProfileHeader() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let navigate = use_navigate();

    let user_info = move || current_user.get().map(|user| user.username().to_string());

    view! {
        <div class="flex justify-between items-center mb-6">
            <div class="flex flex-col items-center space-y-4 flex-1">
                {move || user_info().map(|username| {
                    let text = format!("Профиль {}", username);
                    view! {
                        <>
                            <Heading level=HeadingLevel::H1>
                                {text}
                            </Heading>
                        </>
                    }
                })}
            </div>
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
