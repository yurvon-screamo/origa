mod add_drawer;
mod add_drawer_handlers;
mod add_drawer_state;
mod content;
mod counter_card_item;
mod counter_item;
mod counter_list;
mod detail;
mod header;

pub use content::CountersContent;
pub use detail::CountersDetail;
pub use header::CountersHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Counters() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="counters-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="counters-card">
                <CountersHeader refresh_trigger=refresh_trigger />
                <CountersContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
