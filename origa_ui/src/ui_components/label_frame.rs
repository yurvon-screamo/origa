use leptos::prelude::*;

#[component]
pub fn LabelFrame(
    #[prop(optional, into)] _class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    _children: Children,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div data-testid=test_id_val class=move || format!("label-frame {}", _class.get())>
            {_children()}
        </div>
    }
}
