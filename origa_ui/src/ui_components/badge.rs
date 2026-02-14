use leptos::prelude::*;

#[component]
pub fn Badge(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base_class = "badge";
    let full_class = format!("{} {}", base_class, class);

    view! {
        <span class=full_class>
            {children()}
        </span>
    }
}
