use leptos::prelude::*;

#[component]
pub fn Badge(
    #[prop(optional, into)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <span class=move || format!("badge {}", class.get())>
            {children()}
        </span>
    }
}
