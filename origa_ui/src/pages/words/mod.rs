mod add_word_modal;
mod add_words_preview_modal;
mod add_words_preview_modal_handlers;
mod add_words_preview_modal_state;
mod analyzed_word_item;
mod content;
mod header;
mod image_input_stage;
mod vocabulary_card_item;

pub use content::WordsContent;
pub use header::WordsHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Words() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <WordsHeader refresh_trigger=refresh_trigger />
                <WordsContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
