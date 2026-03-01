use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use origa::domain::Rating;

#[component]
pub fn RatingButtons(
    on_rate: Callback<Rating>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-4 gap-2 mt-6">
            <Button
                variant=Signal::derive(|| ButtonVariant::Default)
                class=Signal::derive(|| "".to_string())
                disabled
                on_click=Callback::new(move |_| on_rate.run(Rating::Again))
            >
                "Не знаю" <span class="hidden sm:inline">"[1]"</span>
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Default)
                class=Signal::derive(|| "".to_string())
                disabled
                on_click=Callback::new(move |_| on_rate.run(Rating::Hard))
            >
                "Плохо" <span class="hidden sm:inline">"[2]"</span>
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Olive)
                class=Signal::derive(|| "".to_string())
                disabled
                on_click=Callback::new(move |_| on_rate.run(Rating::Good))
            >
                "Знаю" <span class="hidden sm:inline">"[3]"</span>
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Filled)
                class=Signal::derive(|| "".to_string())
                disabled
                on_click=Callback::new(move |_| on_rate.run(Rating::Easy))
            >
                "Идеально" <span class="hidden sm:inline">"[4]"</span>
            </Button>
        </div>
    }
}
