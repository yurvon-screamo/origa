use leptos::prelude::*;

#[component]
pub fn Radio(
    #[prop(optional, into)] name: String,
    #[prop(optional, into)] value: String,
    #[prop(into)] checked: Signal<bool>,
    #[prop(optional)] disabled: bool,
    #[prop(optional, into)] label: String,
) -> impl IntoView {
    view! {
        <label class="radio-container">
            <input
                type="radio"
                name=name
                value=value
                checked=checked.get()
                disabled=disabled
            />
            <span class="radio-box"></span>
            <span>{label}</span>
        </label>
    }
}
