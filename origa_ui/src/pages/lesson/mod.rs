mod card_type;
mod complete_screen;
mod content;
mod grammar_info_badge;
mod header;
mod kanji_card_details;
mod keyboard_handler;
mod lesson_card;
mod lesson_card_answer;
mod lesson_card_container;
mod lesson_card_header;
mod lesson_card_question;
mod lesson_progress;
mod lesson_progress_view;
mod lesson_state;
mod on_dont_know;
mod on_quiz_select;
mod on_rate;
mod on_yesno_select;
mod phrase_card;
mod quiz_card;
mod quiz_card_header;
mod quiz_options;
mod quiz_result;
mod quiz_result_display;
mod rating_buttons;
mod rating_buttons_view;
mod writing_card;
mod yesno_card_view;

pub use content::LessonContent;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Lesson() -> impl IntoView {
    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="lesson-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="lesson-card" class="px-4 py-8">
                <LessonContent />
            </CardLayout>
        </PageLayout>
    }
}
