mod action_buttons;
mod add_kanji_modal;
mod add_kanji_modal_handlers;
mod add_kanji_modal_state;
mod content;
mod drawing_drawer;
mod error_alert;
mod header;
mod kanji_card_item;
mod kanji_item;
mod kanji_list;
mod level_selector;
mod selected_count;

pub use content::KanjiContent;
pub use drawing_drawer::DrawingDrawer;
pub use header::KanjiHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Kanji() -> impl IntoView {
    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <KanjiHeader />
                <KanjiContent />
            </CardLayout>
        </PageLayout>
    }
}
