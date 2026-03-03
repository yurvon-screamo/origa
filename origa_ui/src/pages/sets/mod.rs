mod content;
mod header;
mod import_set_preview_modal;
mod import_set_preview_modal_handlers;
mod import_set_preview_modal_state;
mod set_card;
mod set_word_item;
mod sets_level_group;
mod sets_type_group;
mod types;

pub use content::SetsContent;
pub use header::SetsHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Sets() -> impl IntoView {
    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <SetsHeader />
                <SetsContent />
            </CardLayout>
        </PageLayout>
    }
}
