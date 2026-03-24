use crate::ui_components::{Card, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn IntroStep() -> impl IntoView {
    view! {
        <div class="intro-step">
            <div class="text-center mb-8">
                <Text size=TextSize::Large variant=TypographyVariant::Primary>
                    "Добро пожаловать в Origa!"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        "Origa помогает изучать японский язык с интервальными повторениями"
                    </Text>
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4 mt-6">
                <Card class=Signal::derive(|| "text-center p-4".to_string())>
                    <div class="text-4xl mb-2">"📚"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary>
                        "Словарь"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Vocabulary"
                    </Text>
                </Card>

                <Card class=Signal::derive(|| "text-center p-4".to_string())>
                    <div class="text-4xl mb-2">"漢字"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary>
                        "Иероглифы"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Kanji"
                    </Text>
                </Card>

                <Card class=Signal::derive(|| "text-center p-4".to_string())>
                    <div class="text-4xl mb-2">"📖"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary>
                        "Грамматика"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Grammar"
                    </Text>
                </Card>

                <Card class=Signal::derive(|| "text-center p-4".to_string())>
                    <div class="text-4xl mb-2">"部"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary>
                        "Радикалы"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Radicals"
                    </Text>
                </Card>
            </div>

            <div class="text-center mt-6">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    "Выберите свой уровень и начните обучение"
                </Text>
            </div>
        </div>
    }
}
