use leptos::prelude::*;

#[component]
pub fn Radio(
    #[prop(optional, into)] _name: Signal<String>,
    #[prop(optional, into)] _value: Signal<String>,
    #[prop(into)] _checked: Signal<bool>,
    #[prop(optional, into)] _disabled: Signal<bool>,
    #[prop(optional, into)] _label: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <label class="radio-container" data-testid=test_id_val>
            <input
                type="radio"
                name=move || _name.get()
                value=move || _value.get()
                checked=move || _checked.get()
                disabled=move || _disabled.get()
            />
            <span class="radio-box"></span>
            <span>{move || _label.get()}</span>
        </label>
    }
}
