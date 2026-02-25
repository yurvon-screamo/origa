use crate::ui_components::{TabButton, TabButtonState};
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};

#[component]
pub fn TabBar() -> impl IntoView {
    let location = use_location();
    let navigate = use_navigate();

    let is_home_active = move || location.pathname.get() == "/home";
    let is_words_active = move || location.pathname.get() == "/words";
    let is_kanji_active = move || location.pathname.get() == "/kanji";
    let is_sets_active = move || location.pathname.get() == "/sets";
    let is_grammar_active = move || location.pathname.get() == "/grammar";

    let navigate_home = {
        let navigate = navigate.clone();
        Callback::new(move |_: ()| {
            navigate("/home", Default::default());
        })
    };

    let navigate_words = {
        let navigate = navigate.clone();
        Callback::new(move |_: ()| {
            navigate("/words", Default::default());
        })
    };

    let navigate_kanji = {
        let navigate = navigate.clone();
        Callback::new(move |_: ()| {
            navigate("/kanji", Default::default());
        })
    };

    let navigate_sets = {
        let navigate = navigate.clone();
        Callback::new(move |_: ()| {
            navigate("/sets", Default::default());
        })
    };

    let navigate_grammar = {
        let navigate = navigate;
        Callback::new(move |_: ()| {
            navigate("/grammar", Default::default());
        })
    };

    view! {
        <nav class="fixed bottom-0 left-0 right-0 bg-[var(--bg-primary)] border-t border-[var(--border-color)]">
            <div class="flex justify-around items-center py-2">
                <TabButton
                    icon=Signal::stored('\u{1F3E0}'.to_string())
                    label=Signal::stored("Главная".to_string())
                    state=Signal::derive(move || if is_home_active() { TabButtonState::Active } else { TabButtonState::Inactive })
                    on_click=navigate_home
                />

                <TabButton
                    icon=Signal::stored('\u{1F4DD}'.to_string())
                    label=Signal::stored("Слова".to_string())
                    state=Signal::derive(move || if is_words_active() { TabButtonState::Active } else { TabButtonState::Inactive })
                    on_click=navigate_words
                />

                <TabButton
                    icon=Signal::stored("漢".to_string())
                    label=Signal::stored("Кандзи".to_string())
                    state=Signal::derive(move || if is_kanji_active() { TabButtonState::Active } else { TabButtonState::Inactive })
                    on_click=navigate_kanji
                />

                <TabButton
                    icon=Signal::stored('\u{1F4DA}'.to_string())
                    label=Signal::stored("Наборы".to_string())
                    state=Signal::derive(move || if is_sets_active() { TabButtonState::Active } else { TabButtonState::Inactive })
                    on_click=navigate_sets
                />

                <TabButton
                    icon=Signal::stored("文".to_string())
                    label=Signal::stored("Грамматика".to_string())
                    state=Signal::derive(move || if is_grammar_active() { TabButtonState::Active } else { TabButtonState::Inactive })
                    on_click=navigate_grammar
                />
            </div>
        </nav>
    }
}
