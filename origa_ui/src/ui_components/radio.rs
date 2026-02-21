use leptos::prelude::*;

#[component]
pub fn Radio(
    #[prop(optional, into)] name: Signal<String>,
    #[prop(optional, into)] value: Signal<String>,
    #[prop(into)] checked: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] label: Signal<String>,
) -> impl IntoView {
    view! {
        <label class="radio-container">
            <input
                type="radio"
                name=move || name.get()
                value=move || value.get()
                checked=move || checked.get()
                disabled=move || disabled.get()
            />
            <span class="radio-box"></span>
            <span>{move || label.get()}</span>
        </label>
    }
}
