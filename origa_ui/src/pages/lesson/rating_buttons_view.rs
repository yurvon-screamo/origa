use super::rating_buttons::RatingButtons;
use leptos::prelude::*;
use origa::domain::Rating;

#[component]
pub fn RatingButtonsView(
    on_rate: Callback<Rating>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <RatingButtons on_rate disabled />
    }
}
