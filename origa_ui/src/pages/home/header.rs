use crate::ui_components::{Button, ButtonVariant, DisplayText};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use origa::domain::User;

#[component]
pub fn HomeHeader(current_user: RwSignal<Option<User>>) -> impl IntoView {
    view! {
        <header class="border-b border-[var(--border-color)] bg-[var(--bg-primary)]">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between items-center h-16">
                    <div class="flex items-center">
                        <DisplayText class="font-serif text-2xl font-light tracking-tight">
                            "オリガ"
                        </DisplayText>
                    </div>

                    <div class="flex items-center space-x-4">
                        {move || {
                            current_user.get().map(|user| {
                                let username = user.username().to_string();
                                view! {
                                    <div class="flex items-center space-x-4">
                                        <span class="font-mono text-sm text-[var(--fg-muted)]">
                                            {username}
                                        </span>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let navigate = use_navigate();
                                                navigate("/profile", Default::default());
                                            })
                                        >
                                            "Профиль"
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let current_user = use_context::<RwSignal<Option<User>>>()
                                                    .expect("current_user context not provided");
                                                current_user.set(None);
                                                let navigate = use_navigate();
                                                navigate("/", Default::default());
                                            })
                                        >
                                            "Выйти"
                                        </Button>
                                    </div>
                                }
                            })
                        }}
                    </div>
                </div>
            </div>
        </header>
    }
}
