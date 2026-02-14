use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct TabItem {
    pub id: String,
    pub label: String,
}

#[component]
pub fn Tabs(
    #[prop(into)] tabs: Vec<TabItem>,
    #[prop(optional)] active: RwSignal<String>,
) -> impl IntoView {
    let handle_tab_click = move |id: String| {
        active.set(id.clone());
    };

    view! {
        <div class="tabs">
            <For
                each=move || tabs.clone()
                key=|tab| tab.id.clone()
                children=move |tab| {
                    let tab_id_active = tab.id.clone();
                    let tab_id_click = tab.id.clone();
                    let is_active = move || active.get() == tab_id_active;
                    view! {
                        <button
                            class=format!("tab {}", if is_active() { "active" } else { "" })
                            on:click=move |_| handle_tab_click(tab_id_click.clone())
                        >
                            {tab.label}
                        </button>
                    }
                }
            />
        </div>
    }
}
