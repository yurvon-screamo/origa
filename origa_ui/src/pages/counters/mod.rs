//! Страница счётных суффиксов (issue #415): реестр 76 суффиксов по уровням
//! JLPT со статусом колоды, заведение списком, детальная карточка —
//! отдельным роутом `/counters/:suffix` (паттерн кандзи).

mod add_modal;
mod content;
mod detail;

pub use content::CountersContent;
pub use detail::CountersDetail;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Counters() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="counters-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="counters-card">
                <CountersContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
