use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct TabItem {
    pub id: String,
    pub label: String,
}

#[component]
pub fn Tabs(
    #[prop(into)] tabs: Signal<Vec<TabItem>>,
    #[prop(optional)] active: RwSignal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let tab_test_id = move |tab_id: &str| {
        let base = test_id.get();
        if base.is_empty() {
            None
        } else {
            Some(format!("{}-{}", base, tab_id))
        }
    };

    let handle_tab_click = move |id: String| {
        active.set(id.clone());
    };

    view! {
        <div class="tabs" data-testid=test_id_val>
            <For
                each=move || tabs.get()
                key=|tab| tab.id.clone()
                children=move |tab| {
                    let tab_id_active = tab.id.clone();
                    let tab_id_click = tab.id.clone();
                    let tab_label = tab.label.clone();
                    let tab_id_test = tab.id.clone();
                    let is_active = move || active.get() == tab_id_active;
                    let tab_tst_id = move || tab_test_id(&tab_id_test);
                    view! {
                        <button
                            class=move || format!("tab {}", if is_active() { "active" } else { "" })
                            data-testid=tab_tst_id
                            on:click=move |_| handle_tab_click(tab_id_click.clone())
                        >
                            {tab_label}
                        </button>
                    }
                }
            />
        </div>
    }
}
