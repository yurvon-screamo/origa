use leptos::prelude::*;

#[component]
pub fn Tooltip(
    #[prop(optional, into)] text: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    children: Children,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    view! {
        <div class="tooltip-container" data-testid=test_id_val>
            {children()}
            <div class="tooltip">{move || text.get()}</div>
        </div>
    }
}
