mod action_buttons;
mod add_grammar_modal;
mod add_grammar_modal_handlers;
mod add_grammar_modal_state;
mod content;
mod error_alert;
mod filter;
mod filter_btn;
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
    view! {
        <PageLayout variant=PageLayoutVariant::Centered>
            <CardLayout size=CardLayoutSize::Medium>
                <GrammarHeader />
                <GrammarContent />
            </CardLayout>
        </PageLayout>
    }
}
