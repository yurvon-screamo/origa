use leptos::either::*;
use leptos::prelude::*;

#[derive(Clone, Debug)]

pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
}

#[component]
pub fn Breadcrumbs(
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] _items: Signal<Vec<BreadcrumbItem>>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    let items_with_index = move || {
        _items
            .get()
            .into_iter()
            .enumerate()
            .map(|(idx, item)| (idx, item.label.clone(), item.href.clone()))
            .collect::<Vec<_>>()
    };

    view! {
        <div class="breadcrumbs" data-testid=test_id_val>
            <For
                each=items_with_index
                key=|(_, label, _)| label.clone()
                children=move |(idx, label, href)| {
                    let items_count = _items.get().len();
                    let is_last = idx == items_count - 1;
                    let item_test_id_val = move || {
                        let val = test_id.get();
                        if val.is_empty() { None } else { Some(format!("{}-item-{}", val, idx)) }
                    };
                    view! {
                        <>
                            {if let Some(href) = href {
                                Either::Left(view! { <a data-testid=item_test_id_val href=href>{label}</a> })
                            } else {
                                Either::Right(view! { <span data-testid=item_test_id_val>{label}</span> })
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
