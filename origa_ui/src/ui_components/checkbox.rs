use leptos::prelude::*;

#[component]
pub fn Checkbox(
    #[prop(into)] checked: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] label: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    view! {
        <label class=move || format!("checkbox-container {}", class.get())>
            <input
                type="checkbox"
                checked=move || checked.get()
                disabled=move || disabled.get()
            />
            <span class="checkbox-box"></span>
            <span>{move || label.get()}</span>
        </label>
    }
}
