use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use origa::domain::Rating;

#[component]
pub fn RatingButtons(on_rate: Callback<Rating>) -> impl IntoView {
    view! {
        <div class="grid grid-cols-4 gap-2 mt-6">
            <Button
                variant=Signal::derive(|| ButtonVariant::Default)
                class=Signal::derive(|| "".to_string())
                on_click=Callback::new(move |_| on_rate.run(Rating::Again))
            >
                "Не знаю [1]"
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Default)
                class=Signal::derive(|| "".to_string())
                on_click=Callback::new(move |_| on_rate.run(Rating::Hard))
            >
                "Плохо [2]"
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Olive)
                class=Signal::derive(|| "".to_string())
                on_click=Callback::new(move |_| on_rate.run(Rating::Good))
            >
                "Знаю [3]"
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Filled)
                class=Signal::derive(|| "".to_string())
                on_click=Callback::new(move |_| on_rate.run(Rating::Easy))
            >
                "Идеально [4]"
            </Button>
        </div>
    }
}
