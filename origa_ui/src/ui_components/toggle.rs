use leptos::prelude::*;

#[component]
pub fn Toggle(
    #[prop(into)] checked: Signal<bool>,
    #[prop(optional)] disabled: bool,
    #[prop(optional, into)] label: String,
) -> impl IntoView {
    view! {
        <label class="toggle-container">
            <input
                type="checkbox"
                checked=checked.get()
                disabled=disabled
            />
            <span class="toggle-track"></span>
            <span>{label}</span>
        </label>
    }
}
