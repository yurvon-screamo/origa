use crate::i18n::*;
use crate::pages::profile::LanguageSelector;
use crate::ui_components::{
    Card, FuriganaText, Tag, TagVariant, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::NativeLanguage;

#[component]
pub fn IntroStep(
    selected_language: RwSignal<NativeLanguage>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div data-testid=test_id_val class="intro-step max-w-xl mx-auto text-center">
            <div class="intro-welcome flex flex-col items-center gap-4 mb-8">
                <div class="intro-language-controls flex flex-col sm:flex-row items-center gap-3" data-testid="intro-step-language-bar">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true>
                        {t!(i18n, profile.interface_language)}
                    </Text>
                    <LanguageSelector selected_language=selected_language />
                </div>

                <Text size=TextSize::Large variant=TypographyVariant::Primary test_id="intro-step-title">
                    {t!(i18n, onboarding.intro.title)}
                </Text>
                <Text size=TextSize::Default variant=TypographyVariant::Muted test_id="intro-step-subtitle">
                    {t!(i18n, onboarding.intro.subtitle)}
                </Text>
            </div>

            <div class="feature-grid grid grid-cols-1 sm:grid-cols-3 gap-4 mb-6" data-testid="intro-step-feature-showcase">
                <Card shadow=true class="p-4 text-left" test_id="intro-step-feature-vocabulary">
                    <div class="flex items-center gap-2 mb-3">
                        <Tag variant=Signal::derive(|| TagVariant::Default) test_id=Signal::derive(|| "intro-step-tag-vocabulary".to_string())>
                            {t!(i18n, onboarding.intro.vocabulary)}
                        </Tag>
                        <div class="text-2xl">
                            <FuriganaText text="語彙".to_string() known_kanji=Default::default() />
                        </div>
                    </div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, onboarding.intro.vocabulary_desc)}
                    </Text>
                </Card>

                <Card shadow=true class="p-4 text-left" test_id="intro-step-feature-kanji">
                    <div class="flex items-center gap-2 mb-3">
                        <Tag variant=Signal::derive(|| TagVariant::Olive) test_id=Signal::derive(|| "intro-step-tag-kanji".to_string())>
                            {t!(i18n, onboarding.intro.kanji)}
                        </Tag>
                        <div class="text-2xl">
                            <FuriganaText text="漢字".to_string() known_kanji=Default::default() />
                        </div>
                    </div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, onboarding.intro.kanji_desc)}
                    </Text>
                </Card>

                <Card shadow=true class="p-4 text-left" test_id="intro-step-feature-grammar">
                    <div class="flex items-center gap-2 mb-3">
                        <Tag variant=Signal::derive(|| TagVariant::Terracotta) test_id=Signal::derive(|| "intro-step-tag-grammar".to_string())>
                            {t!(i18n, onboarding.intro.grammar)}
                        </Tag>
                        <div class="text-2xl">
                            <FuriganaText text="文法".to_string() known_kanji=Default::default() />
                        </div>
                    </div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, onboarding.intro.grammar_desc)}
                    </Text>
                </Card>
            </div>

            <Text size=TextSize::Small variant=TypographyVariant::Muted test_id="intro-step-footer">
                {t!(i18n, onboarding.intro.footer)}
            </Text>
        </div>
    }
}
