use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn SelectedCount(count: Signal<usize>) -> impl IntoView {
    move || {
        let c = count.get();
        if c > 0 {
            view! {
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {format!("Выбрано: {} правил", c)}
                </Text>
            }
            .into_any()
        } else {
            ().into_any()
        }
    }
}
