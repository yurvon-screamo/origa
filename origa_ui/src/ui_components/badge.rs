use leptos::prelude::*;

#[component]
pub fn Badge(
    #[prop(optional, into)] _class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    _children: Children,
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
        <span class=move || format!("badge {}", _class.get()) data-testid=test_id_val>
            {_children()}
        </span>
    }
}
