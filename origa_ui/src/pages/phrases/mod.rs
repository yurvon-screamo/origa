mod content;
mod header;
mod phrase_card_item;

pub use content::PhrasesContent;
pub use header::PhrasesHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Phrases() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="phrases-page">
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8" test_id="phrases-card">
                <PhrasesHeader />
                <PhrasesContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
