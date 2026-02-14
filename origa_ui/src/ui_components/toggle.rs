use leptos::prelude::*;

#[component]
pub fn Toggle(
    #[prop(into)] checked: Signal<bool>,
    #[prop(optional)] disabled: bool,
    #[prop(optional, into)] label: String,
    #[prop(optional)] on_change: Option<Callback<leptos::ev::Event>>,
) -> impl IntoView {
    view! {
        <label class="toggle-container">
            <input
                type="checkbox"
                checked=checked.get()
                disabled=disabled
                on:change=move |ev| {
                    if let Some(on_change) = on_change {
                        on_change.run(ev);
                    }
                }
            />
            <span class="toggle-track"></span>
            <span>{label}</span>
        </label>
    }
}
