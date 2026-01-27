use leptos::prelude::*;

#[component]
pub fn BaseCard(
    #[prop(into, optional)] class: Option<String>,
    #[prop(into, optional)] onclick: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let class_value = move || {
        let mut classes = vec!["card".to_string()];
        if let Some(custom_class) = class.as_ref() {
            classes.push(custom_class.clone());
        }
        classes.join(" ")
    };

    view! {
        <div
            class=class_value
            on:click=move |ev| {
                if let Some(handler) = onclick {
                    handler.run(ev);
                }
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn CardActions(
    #[prop(into, optional)] _class: Option<String>,
    children: Children,
) -> impl IntoView {
    view! { <div class="card-actions">{children()}</div> }
}
