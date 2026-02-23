mod complete_screen;
mod content;
mod header;
mod lesson_card;
mod lesson_card_container;
mod lesson_progress;
mod lesson_progress_view;
mod lesson_state;
mod rating_buttons;
mod rating_buttons_view;

pub use content::LessonContent;
pub use header::LessonHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Lesson() -> impl IntoView {
    view! {
        <PageLayout variant=PageLayoutVariant::Centered>
            <CardLayout size=CardLayoutSize::Medium>
                <LessonHeader />
                <LessonContent />
            </CardLayout>
        </PageLayout>
    }
}
