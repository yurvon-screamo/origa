use leptos::prelude::*;

#[component]
pub fn Toggle(
    #[prop(into)] _checked: Signal<bool>,
    #[prop(optional)] _disabled: bool,
    #[prop(optional, into)] _label: String,
    #[prop(optional)] _on_change: Option<Callback<leptos::ev::Event>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <label class="toggle-container" data-testid=test_id_val>
            <input
                type="checkbox"
                checked={move || _checked.get()}
                disabled=_disabled
                on:change=move |ev| {
                    if let Some(on_change) = _on_change {
                        on_change.run(ev);
                    }
                }
            />
            <span class="toggle-track"></span>
            <span>{_label}</span>
        </label>
    }
}
