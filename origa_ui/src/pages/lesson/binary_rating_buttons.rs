use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use origa::domain::Rating;

/// Binary self-assessment row of the lesson: «don't know [1]» (default) on
/// the left, «know [2]» (olive) on the right. The single home of the
/// `grid grid-cols-2 gap-3 mt-4` rating pattern — the FSRS rating, the
/// phrase self-assessment, the audio-recall, example-recall and
/// acquaintance-training rows all render through it with their own labels
/// and test ids.
///
/// Labels and hints are reactive (`Signal<String>`) so a locale switch
/// mid-lesson re-renders them like the view-embedded `t!()` did.
#[component]
pub fn BinaryRatingButtons(
    #[prop(into)] left_label: Signal<String>,
    #[prop(into)] right_label: Signal<String>,
    #[prop(into)] left_test_id: Signal<String>,
    #[prop(into)] right_test_id: Signal<String>,
    on_left: Callback<()>,
    on_right: Callback<()>,
    #[prop(into)] left_hint: Signal<String>,
    #[prop(into)] right_hint: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    /// test id of the grid container itself (optional; empty = no
    /// attribute, the Card/Button convention).
    #[prop(optional, into)]
    container_test_id: Signal<String>,
) -> impl IntoView {
    let container_test_id_val = move || {
        let val = container_test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div class="grid grid-cols-2 gap-3 mt-4" data-testid=container_test_id_val>
            <Button
                variant=Signal::derive(|| ButtonVariant::Default)
                class=Signal::derive(|| "".to_string())
                disabled
                test_id=left_test_id
                on_click=Callback::new(move |_| on_left.run(()))
            >
                {move || left_label.get()} <span class="kbd-hint">{move || left_hint.get()}</span>
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Olive)
                class=Signal::derive(|| "".to_string())
                disabled
                test_id=right_test_id
                on_click=Callback::new(move |_| on_right.run(()))
            >
                {move || right_label.get()} <span class="kbd-hint">{move || right_hint.get()}</span>
            </Button>
        </div>
    }
}

/// Standard FSRS rating pair («Не знаю» / «Знаю») with the canonical
/// `lesson-rating-btn-again` / `lesson-rating-btn-good` test ids — the
/// configuration every normal card shares.
#[component]
pub fn RatingButtons(
    on_rate: Callback<Rating>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let on_rate_stored = StoredValue::new(on_rate);
    view! {
        <BinaryRatingButtons
            left_label=Signal::derive(move || t_string!(i18n, lesson.dont_know_rating).to_string())
            right_label=Signal::derive(move || t_string!(i18n, lesson.know).to_string())
            left_test_id="lesson-rating-btn-again"
            right_test_id="lesson-rating-btn-good"
            left_hint="[1]"
            right_hint="[2]"
            disabled
            on_left=Callback::new(move |_| on_rate_stored.get_value().run(Rating::Again))
            on_right=Callback::new(move |_| on_rate_stored.get_value().run(Rating::Good))
        />
    }
}
