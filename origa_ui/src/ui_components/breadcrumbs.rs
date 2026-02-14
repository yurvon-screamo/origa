use leptos::either::*;
use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
}

#[component]
pub fn Breadcrumbs(#[prop(into)] items: Vec<BreadcrumbItem>) -> impl IntoView {
    let items_count = items.len();
    let items_clone = items.clone();
    view! {
        <div class="breadcrumbs">
            <For
                each={
                    let items_clone = items_clone.clone();
                    move || items_clone.clone()
                }
                key=|item| item.label.clone()
                children=move |item| {
                    let is_last = items_clone.iter().position(|i| i.label == item.label) == Some(items_count - 1);
                    view! {
                        <>
                            {if let Some(href) = item.href {
                                Either::Left(view! { <a href=href>{item.label}</a> })
                            } else {
                                Either::Right(view! { <span>{item.label}</span> })
                            }}
                            {if !is_last {
                                Either::Left(view! { <span>"/"</span> })
                            } else {
                                Either::Right(())
                            }}
                        </>
                    }
                }
            />
        </div>
    }
}
