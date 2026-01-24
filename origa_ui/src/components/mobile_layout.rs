use leptos::children::Children;
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};
use thaw::*;

#[component]
pub fn MobileLayout(children: Children) -> impl IntoView {
    view! {
        <div class="mobile-layout">
            <div class="top-bar">
                <h1>"Origa"</h1>
            </div>

            <main class="main-content">
                {children()}
            </main>

            <BottomNavigation />
        </div>
    }
}

#[component]
pub fn BottomNavigation() -> impl IntoView {
    let navigate = use_navigate();
    let navigate_overview = navigate.clone();
    let navigate_learn = navigate.clone();
    let navigate_vocabulary = navigate.clone();
    let navigate_kanji = navigate.clone();
    let navigate_profile = navigate.clone();
    let location = use_location();

    view! {
        <div class="bottom-nav">
            <button
                class="nav-item"
                class:active=move || location.pathname.get() == "/"
                on:click=move |_| navigate_overview("/", Default::default())
            >
                <Icon icon=icondata::AiDashboardOutlined />
                <span>"Обзор"</span>
            </button>
            <button
                class="nav-item"
                class:active=move || location.pathname.get() == "/learn"
                on:click=move |_| navigate_learn("/learn", Default::default())
            >
                <Icon icon=icondata::AiBookOutlined />
                <span>"Учить"</span>
            </button>
            <button
                class="nav-item"
                class:active=move || location.pathname.get().starts_with("/vocabulary")
                on:click=move |_| navigate_vocabulary("/vocabulary", Default::default())
            >
                <Icon icon=icondata::AiFontSizeOutlined />
                <span>"Словарь"</span>
            </button>
            <button
                class="nav-item"
                class:active=move || location.pathname.get().starts_with("/kanji")
                on:click=move |_| navigate_kanji("/kanji", Default::default())
            >
                <Icon icon=icondata::AiFontSizeOutlined />
                <span>"Кандзи"</span>
            </button>
            <button
                class="nav-item"
                class:active=move || location.pathname.get() == "/profile"
                on:click=move |_| navigate_profile("/profile", Default::default())
            >
                <Icon icon=icondata::AiUserOutlined />
                <span>"Профиль"</span>
            </button>
        </div>
    }
}
