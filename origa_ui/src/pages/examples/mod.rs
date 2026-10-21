mod content;
pub(crate) mod example_card_item;
#[cfg(all(target_arch = "wasm32", test))]
mod examples_wasm_tests;
mod lazy_details;

pub use content::ExamplesContent;

use super::shared::card_list_page_header;
use crate::i18n::{td_string, use_i18n};
use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Examples() -> impl IntoView {
    let i18n = use_i18n();
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="examples-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="examples-card">
                {card_list_page_header(
                    "examples",
                    move || i18n.get_keys().examples().title().inner().to_string(),
                    move || td_string!(i18n.get_locale(), examples.hint).to_string(),
                )}
                <ExamplesContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
