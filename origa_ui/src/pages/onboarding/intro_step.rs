use std::collections::HashSet;

use crate::i18n::*;
use crate::ui_components::{Card, FuriganaText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn IntroStep(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
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
                    {t!(i18n, onboarding.intro.title)}
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-subtitle".to_string())>
                        {t!(i18n, onboarding.intro.subtitle)}
                    </Text>
                </div>

                <div class="mt-2">
                      <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-hint".to_string())>
                          {t!(i18n, onboarding.intro.hint)}
                      </Text>
                  </div>
            </div>

            <div class="grid grid-cols-3 gap-4 mt-6">
                <Card class="text-center p-4" test_id=Signal::derive(|| "intro-step-card-vocabulary".to_string())>
                    <div class="text-4xl mb-2">
                        <FuriganaText text="語彙".to_string() known_kanji=empty_kanji_1 />
                    </div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-vocabulary-title".to_string())>
                        {t!(i18n, onboarding.intro.vocabulary)}
                    </Text>
                </Card>

                <Card class="text-center p-4" test_id=Signal::derive(|| "intro-step-card-kanji".to_string())>
                    <div class="text-4xl mb-2">
                        <FuriganaText text="漢字".to_string() known_kanji=empty_kanji_2 />
                    </div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-kanji-title".to_string())>
                        {t!(i18n, onboarding.intro.kanji)}
                    </Text>
                </Card>

                <Card class="text-center p-4" test_id=Signal::derive(|| "intro-step-card-grammar".to_string())>
                    <div class="text-4xl mb-2">
                        <FuriganaText text="文法".to_string() known_kanji=empty_kanji_3 />
                    </div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary test_id=Signal::derive(|| "intro-step-grammar-title".to_string())>
                        {t!(i18n, onboarding.intro.grammar)}
                    </Text>
                </Card>
            </div>

            <div class="text-center mt-6">
                <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "intro-step-footer".to_string())>
                    {t!(i18n, onboarding.intro.footer)}
                </Text>
            </div>
        </div>
    }
}
