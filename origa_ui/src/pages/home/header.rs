use crate::ui_components::{
    Avatar, Button, ButtonVariant, DisplayText, Tag, TagVariant, Text, TextSize, TypographyVariant,
};
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
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    let test_id_jlpt = derive_test_id(test_id, "jlpt");
    let test_id_avatar = derive_test_id(test_id, "avatar");
    let test_id_words = derive_test_id(test_id, "words");
    let test_id_grammar = derive_test_id(test_id, "grammar");
    let test_id_kanji = derive_test_id(test_id, "kanji");
    let test_id_greeting = derive_test_id(test_id, "greeting");

    let current_level = move || {
        current_user
            .get()
            .map(|u| u.jlpt_progress().current_level().code().to_string())
    };

    let username = move || current_user.get().map(|u| u.username().to_string());

    let initials = move || current_user.get().map(|u| u.username().to_uppercase());

    view! {
        <header class="border-b border-[var(--border-dark)] bg-[var(--bg-cream)]" data-testid=test_id_val>
            <div class="px-4 sm:px-6 lg:px-8 py-3">
                <div class="flex items-center justify-between min-h-16">
                    <div class="flex items-center space-x-4">
                        <DisplayText class="font-serif text-2xl font-light tracking-tight whitespace-nowrap">
                            "オリガ"
                        </DisplayText>
                        <Show when=move || username().is_some()>
                            <Text
                                size=TextSize::Small
                                variant=TypographyVariant::Muted
                                class="hidden md:inline"
                                test_id=test_id_greeting
                            >
                                {move || username().map(|name| format!("Добро пожаловать, {}!", name))}
                            </Text>
                        </Show>
                    </div>

                    // Навигация скрыта на mobile (Drawer в Task-005)
                    <div class="hidden md:flex items-center space-x-4">
                        {move || {
                            current_user.get().map(|_| {
                                view! {
                                    <div class="flex items-center gap-4">
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

                    <div class="flex items-center space-x-3">
                        {move || {
                            current_level().map(|level| {
                                view! {
                                    <Tag variant=TagVariant::Olive test_id=test_id_jlpt>
                                        {level}
                                    </Tag>
                                }
                            })
                        }}

                        {move || {
                            initials().map(|init| {
                                let init_for_avatar = init.clone();
                                view! {
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        test_id=test_id_avatar
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            let navigate = use_navigate();
                                            navigate("/profile", Default::default());
                                        })
                                    >
                                        <Avatar initials=Signal::derive(move || init_for_avatar.clone()) />
                                    </Button>
                                }
                            })
                        }}
                    </div>
                </div>
            </div>
        </header>
    }
}

fn derive_test_id(base: Signal<String>, suffix: &str) -> Signal<String> {
    let suffix = suffix.to_string();
    Signal::derive(move || {
        let val = base.get();
        if val.is_empty() {
            format!("home-{}", suffix)
        } else {
            format!("{}-{}", val, suffix)
        }
    })
}
