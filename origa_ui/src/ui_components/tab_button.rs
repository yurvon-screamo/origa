use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum TabButtonState {
    #[default]
    Inactive,

    Active,
}

#[component]
pub fn TabButton(
    #[prop(into)] _icon: Signal<String>,
    #[prop(into)] _label: Signal<String>,
    #[prop(optional, into)] _state: Signal<TabButtonState>,
    #[prop(optional)] _on_click: Option<Callback<()>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let class_str = move || match _state.get() {
        TabButtonState::Active => "flex flex-col items-center text-[var(--accent-olive)]",
        TabButtonState::Inactive => "flex flex-col items-center text-[var(--fg-muted)]",
    };

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    view! {
        <button
            class=class_str
            on:click=move |_| {
                if let Some(cb) = _on_click {
                    cb.run(());
                }
            }
            data-testid=test_id_val
        >
            <span class="text-xl">{move || _icon.get()}</span>
            <span class="text-xs mt-1">{move || _label.get()}</span>
        </button>
    }
}
