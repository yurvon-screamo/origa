use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use origa::domain::Rating;

#[component]
pub fn PhraseRatingButtons(
    on_rate: Callback<Rating>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let base_test_id = move || test_id.get();

    view! {
        <div class="grid grid-cols-2 gap-3 mt-4">
            <Button
                variant=Signal::derive(|| ButtonVariant::Default)
                class=Signal::derive(|| "".to_string())
                disabled
                test_id=Signal::derive(move || format!("{}-did-not-understand", base_test_id()))
                on_click=Callback::new(move |_| on_rate.run(Rating::Again))
            >
                {t!(i18n, lesson.did_not_understand)} <span class="hidden sm:inline" style="color: var(--fg-light)">"[1]"</span>
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Olive)
                class=Signal::derive(|| "".to_string())
                disabled
                test_id=Signal::derive(move || format!("{}-understood", base_test_id()))
                on_click=Callback::new(move |_| on_rate.run(Rating::Good))
            >
                {t!(i18n, lesson.understood)} <span class="hidden sm:inline" style="color: var(--fg-light)">"[2]"</span>
            </Button>
        </div>
    }
}
