mod add_word_modal;
mod content;
mod filter;
mod filter_btn;
mod header;
mod vocabulary_card_item;

pub use content::WordsContent;
pub use header::WordsHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Words() -> impl IntoView {
    view! {
        <PageLayout variant=PageLayoutVariant::Centered>
            <CardLayout size=CardLayoutSize::Medium>
                <WordsHeader />
                <WordsContent />
            </CardLayout>
        </PageLayout>
    }
}
