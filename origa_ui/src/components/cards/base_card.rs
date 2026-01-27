use leptos::prelude::*;

#[component]
pub fn BaseCard(
    #[prop(into, optional)] class: Option<AttributeValue>,
    #[prop(into, optional)] onclick: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let class_value = move || {
        let mut classes = vec!["card".to_string()];
        if let Some(custom_class) = class.as_ref() {
            classes.push(custom_class.to_string());
        }
        classes.join(" ")
    };
    
    view! {
        <div 
            class=class_value
            on:click=move |ev| {
                if let Some(handler) = onclick {
                    handler.call(ev);
                }
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn CardHeader(
    title: String,
    #[prop(optional)] subtitle: Option<String>,
    #[prop(optional)] actions: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="card-header">
            <div>
                <h3 class="card-title">{title}</h3>
                {subtitle.map(|sub| view! {
                    <p class="card-subtitle">{sub}</p>
                })}
            </div>
            
            {actions.map(|actions_children| view! {
                <div class="flex gap-sm">
                    {actions_children()}
                </div>
            })}
        </div>
    }
}

#[component]
pub fn CardContent(
    #[prop(into, optional)] class: Option<AttributeValue>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="card-content">
            {children()}
        </div>
    }
}

#[component]
pub fn CardActions(
    #[prop(into, optional)] class: Option<AttributeValue>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="card-actions">
            {children()}
        </div>
    }
}