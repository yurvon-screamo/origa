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
            if shadow.get() {
                format!("{} {} {}", base_class, shadow_class, class.get())
            } else {
                format!("{} {}", base_class, class.get())
            }
        }>
            {children()}
        </div>
    }
}
