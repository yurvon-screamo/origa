mod acquaintance_keyboard;
mod acquaintance_slides;
mod acquaintance_state;
mod acquaintance_view;
mod answer_display;
mod audio_recall_card;
mod binary_rating_buttons;
mod card_text;
pub mod card_type;
pub(crate) mod complete_screen;
mod content;
mod counter_bindings_session;
mod counter_readings_table;
mod dont_know_button;
mod empty_state_view;
mod example_recall_card;
mod example_selection;
mod grammar_details_expand;
mod grammar_example;
mod grammar_info_badge;
mod hand_progress_strip;
mod header;
mod kanji_card_data;
mod kanji_card_details;
mod keyboard_handler;
mod lesson_card;
mod lesson_card_answer;
mod lesson_card_container;
mod lesson_card_header;
mod lesson_card_question;
mod lesson_card_renderer;
mod lesson_card_shell;
mod lesson_progress;
mod lesson_state;
#[cfg(all(target_arch = "wasm32", test))]
mod lesson_wasm_tests;
mod na_adjective_helper;
mod next_card_button;
mod on_audio_select;
mod on_dont_know;
mod on_quiz_select;
mod on_quiz_submit;
mod on_quiz_toggle;
mod on_rate;
mod on_yesno_select;
pub(crate) mod phrase_card;
mod phrase_cleanup;
pub(crate) mod pos_label;
mod quiz_card;
pub(crate) mod quiz_feedback_strip;
pub(crate) mod quiz_ledger;
pub(crate) mod quiz_ledger_labels;
mod quiz_options;
mod quiz_options_multi;
mod quiz_result;
mod reveal_answer_button;
mod training_answer;
mod training_front;
mod training_view;
pub mod word_example_line;
mod writing_card;
mod yesno_card_view;

pub use content::LessonContent;
pub use lesson_state::LessonContext;

use leptos::prelude::*;

/// Shared Tailwind class for every lesson card view. Pins a stable minimum
/// height in `svh` (small viewport height — stable in both Tauri WebView and
/// dev browser, unlike `dvh` which floats on URL bar collapse) plus `grow` to
/// fill the available `lesson-content` rectangle. See ADR-031.
pub(in crate::pages::lesson) const LESSON_CARD_CLASS: &str =
    "p-4 sm:p-6 min-h-[60svh] sm:min-h-[70svh] flex flex-col grow";

/// Quiz-family cards opt out of the ADR-031 stable floor (owner decision,
/// task U0IjxQ8I2d): the card hugs its content, so the options grid no
/// longer floats above a stretch void. The height now varies between card
/// types by design — the floor's anti-jump guarantee stays with
/// `LESSON_CARD_CLASS` for the other views.
pub(in crate::pages::lesson) const LESSON_CARD_CLASS_COMPACT: &str = "p-4 sm:p-6 flex flex-col";

#[component]
pub fn Lesson() -> impl IntoView {
    view! {
        <div class="flex-1 flex flex-col py-4" data-testid="lesson-page">
            <div class="min-w-[min(90vw,1280px)] max-w-[min(90vw,1280px)] mx-auto px-1 sm:px-2 flex-1 flex flex-col min-h-0" data-testid="lesson-card">
                <LessonContent />
            </div>
        </div>
    }
}
