use super::card_status::CardStatus;
use crate::ui_components::{Tag, TagVariant};
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Filter {
    #[default]
    All,
    New,
    Hard,
    InProgress,
    Learned,
}

impl Filter {
    pub fn label(&self) -> &'static str {
        match self {
            Filter::All => "Все",
            Filter::New => "Новые",
            Filter::Hard => "Сложные",
            Filter::InProgress => "В процессе",
            Filter::Learned => "Изученные",
        }
    }

    pub fn matches(&self, status: CardStatus) -> bool {
        match self {
            Filter::All => true,
            Filter::New => status == CardStatus::New,
            Filter::Hard => status == CardStatus::Hard,
            Filter::InProgress => status == CardStatus::InProgress,
            Filter::Learned => status == CardStatus::Learned,
        }
    }
}

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
