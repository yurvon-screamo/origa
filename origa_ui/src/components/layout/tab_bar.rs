use leptos::prelude::*;
use thaw::*;

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
            class=format!(
                "tab-button {}",
                if is_active.get() { "tab-button-active" } else { "" }
            )
            on:click=move |_| on_click.run(())
        >
            <span class="tab-icon">{icon}</span>
            <span class="tab-label">{label}</span>
        </button>
    }
}

#[component]
pub fn TabBar(active_tab: Signal<String>) -> impl IntoView {
    let navigate = leptos_router::use_navigate();
    
    let handle_tab_click = move |tab: &'static str, path: &'static str| {
        let navigate = navigate.clone();
        Callback::new(move |_| {
            navigate(path, Default::default());
        })
    };
    
    view! {
        <nav class="tab-bar">
            <TabButton 
                icon="🏠" 
                label="Главная" 
                tab="dashboard"
                active_tab 
                on_click=handle_tab_click("dashboard", "/dashboard") />
            <TabButton 
                icon="📚" 
                label="Слова" 
                tab="vocabulary"
                active_tab 
                on_click=handle_tab_click("vocabulary", "/vocabulary") />
            <TabButton 
                icon="🈁" 
                label="Кандзи" 
                tab="kanji"
                active_tab 
                on_click=handle_tab_click("kanji", "/kanji") />
            <TabButton 
                icon="📝" 
                label="Грамматика" 
                tab="grammar"
                active_tab 
                on_click=handle_tab_click("grammar", "/grammar") />
            <TabButton 
                icon="👤" 
                label="Профиль" 
                tab="profile"
                active_tab 
                on_click=handle_tab_click("profile", "/profile") />
        </nav>
    }
}