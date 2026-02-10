use crate::components::layout::tab_bar::TabBar;
use leptos::prelude::*;

#[component]
pub fn AppLayout(
    children: Children,
    #[prop(optional)] active_tab: Option<String>,
) -> impl IntoView {
    let current_tab = Signal::derive(move || {
        active_tab
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "dashboard".to_string())
    });

    view! {
        <div class="mobile-container">
            <main class="page">{children()}</main>

            // Bottom tab navigation (mobile only)
            <TabBar active_tab=current_tab />
        </div>
    }
}

#[component]
pub fn PageHeader(
    #[prop(into)] title: Signal<String>,
    #[prop(optional)] subtitle: Option<String>,
    #[prop(optional)] show_back: Option<bool>,
    #[prop(optional)] back_action: Option<Callback<()>>,
) -> impl IntoView {
    let handle_back = move |_| {
        if let Some(action) = back_action {
            action.run(());
        } else if show_back.unwrap_or(false)
            && let Some(window) = web_sys::window()
        {
            let _ = window.location().set_href("/");
        }
    };

    view! {
        <header class="page-header">
            <div class="flex items-center">
                {show_back
                    .unwrap_or(false)
                    .then(|| {
                        view! {
                            <button
                                class="icon-button me-3"
                                on:click=handle_back
                                aria-label="Назад"
                            >
                                "←"
                            </button>
                        }
                    })}

                <div class="flex flex-col">
                    <h1 class="page-title">{move || title.get()}</h1>
                    {subtitle.map(|sub| view! { <p class="page-subtitle">{sub}</p> })}
                </div>
            </div>
        </header>
    }
}
