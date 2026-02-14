use leptos::prelude::*;

#[component]
pub fn Card(
    #[prop(optional, into)] class: String,
    #[prop(optional)] shadow: bool,
    children: Children,
) -> impl IntoView {
    let base_class = "card";
    let shadow_class = "card-shadow";
    let full_class = if shadow {
        format!("{} {} {}", base_class, shadow_class, class)
    } else {
        format!("{} {}", base_class, class)
    };

    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}
