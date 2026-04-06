mod add_kanji_modal;
mod add_kanji_modal_handlers;
mod add_kanji_modal_state;
mod content;
mod drawing_drawer;
mod header;
mod kanji_card_item;
mod kanji_item;
mod kanji_list;

pub use content::KanjiContent;
pub use drawing_drawer::DrawingDrawer;
pub use header::KanjiHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Kanji() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="kanji-page">
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8" test_id="kanji-card">
                <KanjiHeader refresh_trigger=refresh_trigger />
                <KanjiContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
