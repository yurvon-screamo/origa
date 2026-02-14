use crate::ui_components::{Card, DisplayText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn HomeContent() -> impl IntoView {
    view! {
        <main class="flex-1">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <Card class="p-6">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true class="mb-4">
                            "Канжи"
                        </Text>
                        <DisplayText class="mb-2">
                            "1,245"
                        </DisplayText>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "изученных символов"
                        </Text>
                    </Card>

                    <Card class="p-6">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true class="mb-4">
                            "Слова"
                        </Text>
                        <DisplayText class="mb-2">
                            "3,821"
                        </DisplayText>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "в словаре"
                        </Text>
                    </Card>

                    <Card class="p-6">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true class="mb-4">
                            "Уровень"
                        </Text>
                        <DisplayText class="mb-2">
                            "N5"
                        </DisplayText>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "текущий прогресс"
                        </Text>
                    </Card>
                </div>

                <div class="mt-12">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true class="mb-6">
                        "Сегодня"
                    </Text>
                    <Card class="p-6">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            "Начните изучение японского языка"
                        </Text>
                    </Card>
                </div>
            </div>
        </main>
    }
}
