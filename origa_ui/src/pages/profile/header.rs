use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use origa::domain::User;

#[component]
pub fn ProfileHeader() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let navigate = use_navigate();
    let navigate_home = navigate.clone();
    let navigate_logout = navigate.clone();

    let user_info = move || current_user.get().map(|user| user.username().to_string());

    view! {
        <div class="flex flex-wrap justify-between items-center gap-4 mb-6">
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
            <div class="flex items-center gap-2">
                <Button
                    variant=ButtonVariant::Ghost
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate_home("/home", Default::default());
                    })
                >
                    "Назад"
                </Button>
                <Button
                    variant=ButtonVariant::Ghost
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        current_user.set(None);
                        navigate_logout("/", Default::default());
                    })
                >
                    "Выйти"
                </Button>
            </div>
        </div>
    }
}
