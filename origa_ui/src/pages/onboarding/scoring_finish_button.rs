use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;

use super::telemetry;

/// The Finish button of the onboarding scoring step.
///
/// Finishing is an async checkpoint (`CompleteOnboardingScoringUseCase` +
/// navigation), so while it runs the button must be disabled and flag its
/// loading state on `data-loading` — the same contract as the Summary step's
/// import button (`onboarding-import`).
#[component]
pub fn ScoringFinishButton(
    is_finishing: RwSignal<bool>,
    on_finish: Callback<()>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        // Declarative Umami attributes live on a wrapper span, not on <Button>:
        // under leptos_macro 0.8.18 an `attr:`-prefixed component attribute
        // renders verbatim (`attr:data-umami-event` lands in the DOM) and the
        // tracker's `[data-umami-event]` selector never matches. The click
        // bubbles to the span, where `closest()` finds the event name.
        // Same event-data properties as the programmatic step events
        // (issue #774 / ADR-054 §5.1), so the whole funnel slices
        // uniformly by locale and device.
        <span data-umami-event="onboarding_complete"
             data-umami-event-locale=locale_code(&i18n.get_locale_untracked())
             data-umami-event-device=telemetry::device()>
        <Button
            variant=ButtonVariant::Olive
            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                on_finish.run(());
            })
            disabled=Signal::derive(move || is_finishing.get())
            attr:data-loading=Signal::derive(move || is_finishing.get().to_string())
            test_id=test_id
        >
            {move || if is_finishing.get() { t!(i18n, onboarding.completing).into_any() } else { t!(i18n, onboarding.finish).into_any() }}
        </Button>
        </span>
    }
}
