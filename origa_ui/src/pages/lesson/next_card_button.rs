use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;

/// Unified "Next" button shown after the user submits an answer, under the
/// pure-manual advance model (ADR-033). Rendered by every lesson card view
/// (single-select quiz, yesno, multi-quiz, phrase) when the parent decides
/// the user is held on the feedback card (`waiting_for_next == true`).
///
/// The parent owns the `Show when=...` condition; this component renders only
/// the button. `test_id` is `lesson-card-next-btn` — distinct from the
/// completion screen's `lesson-next-btn` (which starts the NEXT lesson, not
/// the next card) to avoid Playwright strict-mode ambiguity during the
/// async gap between the last card rate and `is_completed = true`.
#[component]
pub fn NextCardButton(on_next_card: Callback<()>) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <div class="mt-4 flex justify-center">
            <Button
                variant=Signal::derive(|| ButtonVariant::Filled)
                on_click=Callback::new(move |_| on_next_card.run(()))
                test_id=Signal::derive(|| "lesson-card-next-btn".to_string())
            >
                <span>{t!(i18n, lesson.next)}</span>
                <span class="kbd-hint text-[var(--fg-light)]">{t!(i18n, lesson.space_key)}</span>
            </Button>
        </div>
    }
}
