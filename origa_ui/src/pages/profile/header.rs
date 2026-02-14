use crate::ui_components::{Heading, HeadingLevel};
use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn ProfileHeader() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");

    let user_info = move || current_user.get().map(|user| user.username().to_string());

    view! {
        <div class="flex flex-col items-center space-y-4">
            {move || user_info().map(|username| {
                let text = format!("Профиль пользователя {}", username);
                view! {
                    <>
                        <Heading level=HeadingLevel::H1>
                            {text}
                        </Heading>
                    </>
                }
            })}
        </div>
    }
}
