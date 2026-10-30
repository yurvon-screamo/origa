use crate::ui_components::Card;
use leptos::prelude::*;

use super::LESSON_CARD_CLASS;

/// Shared scaffold of a lesson card view: the paper card (stable ADR-031
/// height via `LESSON_CARD_CLASS` by default; the quiz family overrides
/// it with the compact content-hugging class) with a vertically centered
/// body, plus an optional tags row rendered ABOVE the card border
/// (card-type tag, POS, quiz variant, header audio).
///
/// Direction-B scaffold (task 3SekkMiYYW): the card holds ONLY content.
/// Optional sibling zones below the card keep the reading order
/// question → feedback → action: `feedback` renders the quiz answer
/// ledger strip (aged paper band), `footer` renders the action buttons.
/// Views with their own below-card blocks (example recall rating row)
/// may keep their own scaffold — wrapping them here would break the
/// flex chain that stretches the card to the lesson viewport.
#[component]
pub fn LessonCardShell(
    /// Tags row content, rendered above the Card (outside its border).
    #[prop(optional)]
    tags: Option<ChildrenFn>,
    /// Feedback zone below the card (quiz answer ledger strip).
    #[prop(optional)]
    feedback: Option<ChildrenFn>,
    /// Action zone below the card and the feedback strip (buttons).
    #[prop(optional)]
    footer: Option<ChildrenFn>,
    /// Card class override: the quiz family passes
    /// `LESSON_CARD_CLASS_COMPACT` (no ADR-031 floor — owner decision);
    /// everything else keeps the shared stable-height default.
    #[prop(default = Signal::derive(|| LESSON_CARD_CLASS.to_string()), into)]
    card_class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="flex flex-col">
            {tags.map(|tags| tags())}
            <Card class=Signal::derive(move || card_class.get()) shadow=true test_id=Signal::derive(move || test_id.get())>
                <div class="flex-1 flex flex-col justify-center">
                    {children()}
                </div>
            </Card>
            {feedback.map(|feedback| feedback())}
            {footer.map(|footer| footer())}
        </div>
    }
}
