use crate::ui_components::{Card, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn IntroStep(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    view! {
        <div class="intro-step" data-testid=test_id_val>
            <div class="text-center mb-8">
                <Text size=TextSize::Large variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-title".to_string())>
                    "Настроим обучение!"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-subtitle".to_string())>
                        "Сейчас подберём наборы слов под ваш уровень и опыт. Это займёт около 2 минут."
                    </Text>
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4 mt-6">
                <Card class=Signal::derive(|| "text-center p-4".to_string()) test_id=Signal::derive(|| "intro-step-card-vocabulary".to_string())>
                    <div class="text-4xl mb-2">"📚"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-vocabulary-title".to_string())>
                        "Словарь"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-vocabulary-desc".to_string())>
                        "Слова из учебников и курсов"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-vocabulary-en".to_string())>
                        "Vocabulary"
                    </Text>
                </Card>

                <Card class=Signal::derive(|| "text-center p-4".to_string()) test_id=Signal::derive(|| "intro-step-card-kanji".to_string())>
                    <div class="text-4xl mb-2">"漢字"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-kanji-title".to_string())>
                        "Иероглифы"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-kanji-desc".to_string())>
                        "Китайские символы для записи"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-kanji-en".to_string())>
                        "Kanji"
                    </Text>
                </Card>

                <Card class=Signal::derive(|| "text-center p-4".to_string()) test_id=Signal::derive(|| "intro-step-card-grammar".to_string())>
                    <div class="text-4xl mb-2">"📖"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-grammar-title".to_string())>
                        "Грамматика"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-grammar-desc".to_string())>
                        "Правила построения предложений"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-grammar-en".to_string())>
                        "Grammar"
                    </Text>
                </Card>

                <Card class=Signal::derive(|| "text-center p-4".to_string()) test_id=Signal::derive(|| "intro-step-card-radicals".to_string())>
                    <div class="text-4xl mb-2">"部"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-radicals-title".to_string())>
                        "Радикалы"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-radicals-desc".to_string())>
                        "Составные части иероглифов"
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-radicals-en".to_string())>
                        "Radicals"
                    </Text>
                </Card>
            </div>

            <div class="text-center mt-6">
                <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-footer".to_string())>
                    "Эти наборы будут импортированы на основе ваших выборов"
                </Text>
            </div>
        </div>
    }
}
