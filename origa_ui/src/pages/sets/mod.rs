mod content;

pub use content::SetsContent;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Sets() -> impl IntoView {
    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <SetsContent />
            </CardLayout>
        </PageLayout>
    }
}
