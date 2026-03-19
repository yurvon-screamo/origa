use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn SelectedCount(count: Signal<usize>) -> impl IntoView {
    view! {
        <Text size=TextSize::Small variant=TypographyVariant::Muted>
            {move || format!("Выбрано: {}", count.get())}
        </Text>
    }
}
