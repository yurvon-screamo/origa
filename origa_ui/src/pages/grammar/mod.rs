mod add_grammar_modal;
mod add_grammar_modal_handlers;
mod add_grammar_modal_state;
mod content;
mod error_alert;
mod grammar_card_item;
mod header;
mod level_selector;
mod rule_item;
mod rules_list;
mod selected_count;

pub use content::GrammarContent;
pub use header::GrammarHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Grammar() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <GrammarHeader refresh_trigger=refresh_trigger />
                <GrammarContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
