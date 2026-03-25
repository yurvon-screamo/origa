use crate::ui_components::{Avatar, Button, ButtonVariant, DisplayText};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use origa::domain::User;

#[component]
pub fn HomeHeader(
    current_user: RwSignal<Option<User>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let test_id_avatar = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "home-avatar".to_string()
        } else {
            format!("{}-avatar", val)
        }
    });

    let test_id_words = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "home-words".to_string()
        } else {
            format!("{}-words", val)
        }
    });

    let test_id_grammar = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "home-grammar".to_string()
        } else {
            format!("{}-grammar", val)
        }
    });

    let test_id_kanji = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "home-kanji".to_string()
        } else {
            format!("{}-kanji", val)
        }
    });

    view! {
        <header class="border-b border-[var(--border-dark)] bg-[var(--bg-cream)]" data-testid=test_id_val>
            <div class="w-full px-4 sm:px-6 lg:px-8">
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
                                        test_id=test_id_avatar
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
                                            test_id=test_id_words
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let navigate = use_navigate();
                                                navigate("/words", Default::default());
                                            })
                                        >
                                            "Слова"
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            test_id=test_id_grammar
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let navigate = use_navigate();
                                                navigate("/grammar", Default::default());
                                            })
                                        >
                                            "Грамматика"
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            test_id=test_id_kanji
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
