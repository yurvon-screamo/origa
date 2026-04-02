use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn SelectedCount(count: Signal<usize>) -> impl IntoView {
    view! {
        <Text size=TextSize::Small variant=TypographyVariant::Muted>
            <span data-testid="kanji-selected-count">{move || format!("Выбрано: {}", count.get())}</span>
        </Text>
    }
}
