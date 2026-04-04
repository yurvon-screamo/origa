use std::collections::HashSet;

use crate::ui_components::{Card, FuriganaText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn IntroStep(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let empty_kanji_1 = HashSet::new();
    let empty_kanji_2 = HashSet::new();
    let empty_kanji_3 = HashSet::new();

    view! {
        <div class="intro-step" data-testid=test_id_val>
            <div class="text-center mb-8">
                <Text size=TextSize::Large variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-title".to_string())>
                    "Настроим обучение!"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-subtitle".to_string())>
                        "Origa — это приложение для изучения японского языка по карточкам с интервальными повторениями."
                    </Text>
                </div>

                <div class="mt-2">
                      <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-subtitle".to_string())>
                          "Давайте подберём наборы карточек под ваш уровень и опыт. Это займёт совсем немного времени."
                      </Text>
                  </div>
            </div>

            <div class="grid grid-cols-3 gap-4 mt-6">
                <Card class="text-center p-4" test_id=Signal::derive(|| "intro-step-card-vocabulary".to_string())>
                    <div class="text-4xl mb-2">
                        <FuriganaText text="語彙".to_string() known_kanji=empty_kanji_1 />
                    </div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-vocabulary-title".to_string())>
                        "Слова"
                    </Text>
                </Card>

                <Card class="text-center p-4" test_id=Signal::derive(|| "intro-step-card-kanji".to_string())>
                    <div class="text-4xl mb-2">
                        <FuriganaText text="漢字".to_string() known_kanji=empty_kanji_2 />
                    </div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-kanji-title".to_string())>
                        "Кандзи"
                    </Text>
                </Card>

                <Card class="text-center p-4" test_id=Signal::derive(|| "intro-step-card-grammar".to_string())>
                    <div class="text-4xl mb-2">
                        <FuriganaText text="文法".to_string() known_kanji=empty_kanji_3 />
                    </div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-grammar-title".to_string())>
                        "Грамматика"
                    </Text>
                </Card>
            </div>

            <div class="text-center mt-6">
                <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-footer".to_string())>
                    "Все это вы сможете изучать в приложении."
                </Text>
            </div>
        </div>
    }
}
