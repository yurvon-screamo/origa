use leptos::prelude::*;

#[component]
pub fn Badge(#[prop(optional, into)] _class: Signal<String>, _children: Children) -> impl IntoView {
    view! {
        <span class=move || format!("badge {}", _class.get())>
            {_children()}
        </span>
    }
}
