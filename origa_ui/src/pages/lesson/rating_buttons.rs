use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use origa::domain::Rating;

#[component]
pub fn RatingButtons(
    on_rate: Callback<Rating>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mt-4">
            <Button
                variant=Signal::derive(|| ButtonVariant::Default)
                class=Signal::derive(|| "".to_string())
                disabled
                test_id=Signal::derive(|| "lesson-rating-btn-again".to_string())
                on_click=Callback::new(move |_| on_rate.run(Rating::Again))
            >
                {t!(i18n, lesson.dont_know_rating)} <span class="hidden sm:inline">"[1]"</span>
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Default)
                class=Signal::derive(|| "".to_string())
                disabled
                test_id=Signal::derive(|| "lesson-rating-btn-hard".to_string())
                on_click=Callback::new(move |_| on_rate.run(Rating::Hard))
            >
                {t!(i18n, lesson.hard)} <span class="hidden sm:inline">"[2]"</span>
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Olive)
                class=Signal::derive(|| "".to_string())
                disabled
                test_id=Signal::derive(|| "lesson-rating-btn-good".to_string())
                on_click=Callback::new(move |_| on_rate.run(Rating::Good))
            >
                {t!(i18n, lesson.know)} <span class="hidden sm:inline">"[3]"</span>
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Filled)
                class=Signal::derive(|| "".to_string())
                disabled
                test_id=Signal::derive(|| "lesson-rating-btn-easy".to_string())
                on_click=Callback::new(move |_| on_rate.run(Rating::Easy))
            >
                {t!(i18n, lesson.perfect)} <span class="hidden sm:inline">"[4]"</span>
            </Button>
        </div>
    }
}
