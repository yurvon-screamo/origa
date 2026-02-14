use leptos::prelude::*;

#[component]
pub fn Checkbox(
    #[prop(into)] checked: Signal<bool>,
    #[prop(optional)] disabled: bool,
    #[prop(optional, into)] label: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let full_class = format!("checkbox-container {}", class);

    view! {
        <label class=full_class>
            <input
                type="checkbox"
                checked=checked.get()
                disabled=disabled
            />
            <span class="checkbox-box"></span>
            <span>{label}</span>
        </label>
    }
}
