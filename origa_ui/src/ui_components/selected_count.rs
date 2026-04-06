use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn SelectedCount(
    count: Signal<usize>,
    #[prop(optional, into)] label: Signal<String>,
) -> impl IntoView {
    move || {
        let c = count.get();
        let label_text = label.get();
        if c > 0 {
            let display = if label_text.is_empty() {
                format!("Выбрано: {}", c)
            } else {
                format!("Выбрано: {} {}", c, label_text)
            };
            view! {
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {display}
                </Text>
            }
            .into_any()
        } else {
            ().into_any()
        }
    }
}
