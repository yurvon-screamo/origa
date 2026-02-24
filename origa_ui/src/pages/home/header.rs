use crate::ui_components::{Avatar, Button, ButtonVariant, DisplayText};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use origa::domain::User;

#[component]
pub fn HomeHeader(current_user: RwSignal<Option<User>>) -> impl IntoView {
    view! {
        <header class="border-b border-[var(--border-color)] bg-[var(--bg-primary)]">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between items-center h-16">
                    <div class="flex items-center space-x-4">
                        {move || {
                            current_user.get().map(|user| {
                                let initials = user.username()
                                    .split_whitespace()
                                    .filter_map(|word| word.chars().next())
                                    .take(2)
                                    .collect::<String>()
                                    .to_uppercase();
                                let greeting = format!("Привет, {}!", user.username());
                                let initials_clone = initials.clone();
                                view! {
                                    <Avatar initials=Signal::derive(move || initials_clone.clone()) />
                                    <DisplayText class="font-serif text-2xl font-light tracking-tight">
                                        "オリガ"
                                    </DisplayText>
                                    <span class="font-mono text-sm text-[var(--fg-muted)]">
                                        {greeting}
                                    </span>
                                }
                            })
                        }}
                    </div>

                    <div class="flex items-center space-x-4">
                        {move || {
                            current_user.get().map(|_| {
                                view! {
                                    <div class="flex items-center space-x-4">
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let navigate = use_navigate();
                                                navigate("/words", Default::default());
                                            })
                                        >
                                            "Слова"
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let navigate = use_navigate();
                                                navigate("/grammar", Default::default());
                                            })
                                        >
                                            "Грамматика"
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let navigate = use_navigate();
                                                navigate("/kanji", Default::default());
                                            })
                                        >
                                            "Кандзи"
                                        </Button>
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
