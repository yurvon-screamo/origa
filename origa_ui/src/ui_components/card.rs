use leptos::prelude::*;

#[component]
pub fn Card(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] shadow: Signal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=move || {
            let base_class = "card";
            let shadow_class = "card-shadow";
            let user_class = class.get();
            let is_interactive = user_class.contains("cursor-pointer") || user_class.contains("interactive");
            let anima_class = if is_interactive { "anima-card-lift" } else { "" };

            if shadow.get() {
                format!("{} {} {} {}", base_class, shadow_class, anima_class, user_class)
            } else {
                format!("{} {} {}", base_class, anima_class, user_class)
            }
        }>
            {children()}
        </div>
    }
}
