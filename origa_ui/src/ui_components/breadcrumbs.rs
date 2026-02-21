use leptos::either::*;
use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
}

#[component]
pub fn Breadcrumbs(#[prop(optional, into)] items: Signal<Vec<BreadcrumbItem>>) -> impl IntoView {
    view! {
        <div class="breadcrumbs">
            <For
                each=move || items.get()
                key=|item| item.label.clone()
                children=move |item| {
                    let items_count = items.get().len();
                    let is_last = items.get().iter().position(|i| i.label == item.label) == Some(items_count - 1);
                    let label = item.label.clone();
                    let href = item.href.clone();
                    view! {
                        <>
                            {if let Some(href) = href {
                                Either::Left(view! { <a href=href>{label}</a> })
                            } else {
                                Either::Right(view! { <span>{label}</span> })
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
