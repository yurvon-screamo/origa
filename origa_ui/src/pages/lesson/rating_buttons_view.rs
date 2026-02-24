use super::rating_buttons::RatingButtons;
use leptos::prelude::*;
use origa::domain::Rating;

#[component]
pub fn RatingButtonsView(on_rate: Callback<Rating>) -> impl IntoView {
    view! {
        <RatingButtons on_rate />
    }
}
