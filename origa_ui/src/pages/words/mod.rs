mod add_word_modal;
mod content;
mod header;
mod vocabulary_card_item;

pub use content::WordsContent;
pub use header::WordsHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Words() -> impl IntoView {
    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <WordsHeader />
                <WordsContent />
            </CardLayout>
        </PageLayout>
    }
}
