use crate::ui_components::{Heading, HeadingLevel, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn LoginHeader() -> impl IntoView {
    view! {
        <div class="text-center mb-10">
            <Heading level=HeadingLevel::H1 variant=TypographyVariant::Primary class="mb-3">
                "オリガ"
            </Heading>
            <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true>
                "Изучение японского языка"
            </Text>
        </div>
    }
}
