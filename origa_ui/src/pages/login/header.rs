use crate::ui_components::{DisplayText, Heading, HeadingLevel, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn LoginHeader() -> impl IntoView {
    view! {
        <div class="text-center mb-10">
            <div class="inline-block mb-6">
                <DisplayText class="w-16 h-16 mx-auto mb-4 flex items-center justify-center text-4xl font-serif text-[var(--accent-olive)]">
                    "桜"
                </DisplayText>
            </div>
            <Heading level=HeadingLevel::H1 variant=TypographyVariant::Primary class="mb-3">
                "オリガ"
            </Heading>
            <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true>
                "Изучение японского языка"
            </Text>
        </div>
    }
}
