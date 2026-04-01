use leptos::prelude::*;

#[component]
pub fn AccordionItem(
    #[prop(optional, into)] _header: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    _children: Children,
) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);

    let toggle = move |_| {
        set_is_open.update(|open| *open = !*open);
    };

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    let test_id_toggle = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(format!("{}-toggle", val))
        }
    };

    view! {
        <div
            class=move || format!("accordion-item {}", if is_open.get() { "active" } else { "" })
            data-testid=test_id_val
        >
            <div
                class="accordion-header"
                data-testid=test_id_toggle
                on:click=toggle
            >
                <span class="accordion-title">{move || _header.get()}</span>
                <span class="accordion-icon"></span>
            </div>
            <div
                class=move || format!("accordion-content accordion-max-height {}", if is_open.get() { "accordion-max-height-open" } else { "" })
            >
                <div class="accordion-body">
                    {_children()}
                </div>
            </div>
        </div>
    }
}
