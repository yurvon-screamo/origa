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
                        "Подберём наборы под ваш уровень и опыт. Это займёт совсем немного времени."
                    </Text>
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4 mt-6">
                <Card class=Signal::derive(|| "text-center p-4".to_string()) test_id=Signal::derive(|| "intro-step-card-vocabulary".to_string())>
                    <div class="text-4xl mb-2">"語彙"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-vocabulary-title".to_string())>
                        "Слова"
                    </Text>
                </Card>

                <Card class=Signal::derive(|| "text-center p-4".to_string()) test_id=Signal::derive(|| "intro-step-card-kanji".to_string())>
                    <div class="text-4xl mb-2">"漢字"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-kanji-title".to_string())>
                        "Кандзи"
                    </Text>
                </Card>

                <Card class=Signal::derive(|| "text-center p-4".to_string()) test_id=Signal::derive(|| "intro-step-card-grammar".to_string())>
                    <div class="text-4xl mb-2">"文法"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-grammar-title".to_string())>
                        "Грамматика"
                    </Text>
                </Card>

                <Card class=Signal::derive(|| "text-center p-4".to_string()) test_id=Signal::derive(|| "intro-step-card-radicals".to_string())>
                    <div class="text-4xl mb-2">"部"</div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-radicals-title".to_string())>
                        "Радикалы"
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
