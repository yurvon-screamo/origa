use leptos::prelude::*;

#[component]
pub fn Card(
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] shadow: Signal<bool>,
    #[prop(optional, into)] borderless: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div class=move || {
            let base_class = if borderless.get() { "card border-none" } else { "card" };
            let shadow_class = "card-shadow";
            let user_class = class.get();
            let is_interactive = user_class.contains("cursor-pointer") || user_class.contains("interactive");
            let anima_class = if is_interactive { "anima-lift" } else { "" };

            if shadow.get() {
                format!("{} {} {} {}", base_class, shadow_class, anima_class, user_class)
            } else {
                format!("{} {} {}", base_class, anima_class, user_class)
            }
        } data-testid=test_id_val>
            {children()}
        </div>
    }
}
