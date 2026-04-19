use crate::i18n::*;
use crate::ui_components::{Avatar, Button, ButtonVariant, DisplayText, derive_test_id};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use origa::domain::User;

#[component]
pub fn HomeHeader(
    current_user: RwSignal<Option<User>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let test_id_avatar = derive_test_id(test_id, "avatar");
    let test_id_words = derive_test_id(test_id, "words");
    let test_id_grammar = derive_test_id(test_id, "grammar");
    let test_id_kanji = derive_test_id(test_id, "kanji");
    let test_id_phrases = derive_test_id(test_id, "phrases");

    let initials = move || current_user.get().map(|u| u.username().to_uppercase());

    view! {
        <header class="border-b border-[var(--border-dark)] bg-[var(--bg-cream)]" data-testid=test_id_val>
            <div class="px-4 sm:px-6 lg:px-8 py-3">
                <div class="flex items-center justify-between min-h-16">
                    <div class="flex items-center space-x-4">
                        <DisplayText class="font-serif text-2xl font-light tracking-tight whitespace-nowrap">
                            "オリガ"
                        </DisplayText>
                    </div>

                    // Навигация скрыта на mobile
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
                                            {t!(i18n, home.words)}
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            test_id=test_id_grammar
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let navigate = use_navigate();
                                                navigate("/grammar", Default::default());
                                            })
                                        >
                                            {t!(i18n, home.grammar)}
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            test_id=test_id_kanji
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let navigate = use_navigate();
                                                navigate("/kanji", Default::default());
                                            })
                                        >
                                            {t!(i18n, home.kanji)}
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            test_id=test_id_phrases
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let navigate = use_navigate();
                                                navigate("/phrases", Default::default());
                                            })
                                        >
                                            {t!(i18n, home.phrases)}
                                        </Button>
                                    </div>
                                }
                            })
                        }}
                    </div>

                    <div class="hidden md:block">
                        {move || {
                            initials().map(|init| {
                                let init_for_avatar = init.clone();
                                view! {
                                    <div class="anima-avatar-hover" on:click=move |_: leptos::ev::MouseEvent| {
                                        let navigate = use_navigate();
                                        navigate("/profile", Default::default());
                                    }>
                                        <Avatar test_id=test_id_avatar initials=Signal::derive(move || init_for_avatar.clone()) />
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
