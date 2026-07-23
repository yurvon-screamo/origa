mod add_word_modal;
mod add_words_preview_modal;
mod add_words_preview_modal_handlers;
mod add_words_preview_modal_state;
mod analyzed_word_item;
mod anki_import_stage;
mod audio_input_stage;
mod content;
mod header;
mod image_input_stage;
mod ocr_file_utils;
mod ocr_processing;
mod vocabulary_card_item;

#[cfg(target_arch = "wasm32")]
pub use audio_input_stage::preload_whisper_model;
pub use content::WordsContent;
pub use header::WordsHeader;
pub use ocr_processing::preload_ocr_model;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Words() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="words-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="words-card">
                <WordsHeader refresh_trigger=refresh_trigger />
                <WordsContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
