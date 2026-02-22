use super::filter::Filter;
use crate::ui_components::{Tag, TagVariant};
use leptos::prelude::*;

#[component]
pub fn FilterBtn<F: Fn() -> usize + Send + 'static>(
    filter: Filter,
    count: F,
    active: RwSignal<Filter>,
) -> impl IntoView {
    let is_active = Memo::new(move |_| active.get() == filter);
    let filter_for_click = filter;

    view! {
        <Tag
            variant=Signal::derive(move || if is_active.get() { TagVariant::Filled } else { TagVariant::Default })
            class=Signal::derive(|| "cursor-pointer".to_string())
            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                active.set(filter_for_click);
            })
        >
            {move || format!("{} ({})", filter.label(), count())}
        </Tag>
    }
}
