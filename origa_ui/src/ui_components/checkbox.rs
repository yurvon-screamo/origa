use leptos::prelude::*;

#[component]
pub fn Checkbox(
    #[prop(into)] checked: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] label: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional)] on_change: Option<Callback<()>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let checkbox_checked = checked;
    let checkbox_disabled = disabled;
    let checkbox_label = label;
    let checkbox_class = class;
    let checkbox_test_id = test_id;

    let test_id_val = move || {
        let val = checkbox_test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <label class=move || format!("checkbox-container {}", checkbox_class.get()) data-testid=test_id_val>
            <input
                type="checkbox"
                checked=move || checkbox_checked.get()
                disabled=move || checkbox_disabled.get()
                on:change=move |_| {
                    if let Some(cb) = on_change {
                        cb.run(());
                    }
                }
            />
            <span class="checkbox-box"></span>
            <span>{move || checkbox_label.get()}</span>
        </label>
    }
}
