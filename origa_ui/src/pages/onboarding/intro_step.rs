use crate::ui_components::{Card, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn IntroStep() -> impl IntoView {
    view! {
        <div class="intro-step">
            <div class="text-center mb-8">
                <Text size=TextSize::Large variant=TypographyVariant::Primary>
                    "Настроим обучение!"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        "Сейчас подберём наборы слов под ваш уровень и опыт. Это займёт около 2 минут."
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
                        "Слова из учебников и курсов"
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
                        "Китайские символы для записи"
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
                        "Правила построения предложений"
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
                        "Составные части иероглифов"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Radicals"
                    </Text>
                </Card>
            </div>

            <div class="text-center mt-6">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    "Эти наборы будут импортированы на основе ваших выборов"
                </Text>
            </div>
        </div>
    }
}
