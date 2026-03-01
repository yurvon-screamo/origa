use crate::ui_components::{Avatar, Button, ButtonVariant, DisplayText};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use origa::domain::User;

#[component]
pub fn HomeHeader(current_user: RwSignal<Option<User>>) -> impl IntoView {
    view! {
        <header class="border-b border-[var(--border-color)] bg-[var(--bg-primary)]">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex flex-wrap justify-between items-center min-h-16 gap-4">
                    <div class="flex items-center space-x-4">
                        {move || {
                            current_user.get().map(|user| {
                                let initials = user.username().to_uppercase();
                                let initials_clone = initials.clone();
                                view! {
                                    <DisplayText class="font-serif text-2xl font-light tracking-tight whitespace-nowrap">
                                        "オリガ"
                                    </DisplayText>
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            let navigate = use_navigate();
                                            navigate("/profile", Default::default());
                                        })
                                    >
                                        <Avatar initials=Signal::derive(move || initials_clone.clone()) />
                                    </Button>
                                }
                            })
                        }}
                    </div>

                    <div class="flex items-center space-x-4">
                        {move || {
                            current_user.get().map(|_| {
                                view! {
                    <div class="flex flex-wrap items-center gap-4">
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
