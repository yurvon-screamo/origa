mod content;
pub(crate) mod example_card_item;
#[cfg(all(target_arch = "wasm32", test))]
mod examples_wasm_tests;
pub(crate) mod header;
mod lazy_details;

pub use content::ExamplesContent;
pub use header::ExamplesHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Examples() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="examples-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="examples-card">
                <ExamplesHeader />
                <ExamplesContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
