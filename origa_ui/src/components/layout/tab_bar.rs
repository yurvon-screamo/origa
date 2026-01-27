use leptos::prelude::*;

#[component]
pub fn TabButton(
    icon: &'static str,
    label: &'static str,
    tab: &'static str,
    active_tab: Signal<String>,
    on_click: Callback<()>,
) -> impl IntoView {
    let is_active = Signal::derive(move || active_tab.get() == tab);

    view! {
        <button
            class=format!("tab-button {}", if is_active.get() { "tab-button-active" } else { "" })
            on:click=move |_| on_click.run(())
        >
            <span class="tab-icon">{icon}</span>
            <span class="tab-label">{label}</span>
        </button>
    }
}

#[component]
pub fn TabBar(active_tab: Signal<String>) -> impl IntoView {
    let handle_dashboard = Callback::new(move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/dashboard");
        }
    });

    let handle_vocabulary = Callback::new(move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/vocabulary");
        }
    });

    let handle_kanji = Callback::new(move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/kanji");
        }
    });

    let handle_grammar = Callback::new(move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/grammar");
        }
    });

    let handle_profile = Callback::new(move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/profile");
        }
    });

    view! {
        <nav class="tab-bar">
            <TabButton
                icon="🏠"
                label="Главная"
                tab="dashboard"
                active_tab
                on_click=handle_dashboard
            />
            <TabButton
                icon="📚"
                label="Слова"
                tab="vocabulary"
                active_tab
                on_click=handle_vocabulary
            />
            <TabButton
                icon="🈁"
                label="Кандзи"
                tab="kanji"
                active_tab
                on_click=handle_kanji
            />
            <TabButton
                icon="📝"
                label="Грамматика"
                tab="grammar"
                active_tab
                on_click=handle_grammar
            />
            <TabButton
                icon="👤"
                label="Профиль"
                tab="profile"
                active_tab
                on_click=handle_profile
            />
        </nav>
    }
}
