use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum TabButtonState {
    #[default]
    Inactive,
    Active,
}

#[component]
pub fn TabButton(
    #[prop(into)] icon: Signal<String>,
    #[prop(into)] label: Signal<String>,
    #[prop(optional, into)] state: Signal<TabButtonState>,
    #[prop(optional)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    let class_str = move || match state.get() {
        TabButtonState::Active => "flex flex-col items-center text-[var(--accent-olive)]",
        TabButtonState::Inactive => "flex flex-col items-center text-[var(--fg-muted)]",
    };

    view! {
        <button
            class=class_str
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(());
                }
            }
        >
            <span class="text-xl">{move || icon.get()}</span>
            <span class="text-xs mt-1">{move || label.get()}</span>
        </button>
    }
}
