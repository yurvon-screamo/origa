use crate::i18n::*;
use crate::pages::profile::LanguageSelector;
use crate::ui_components::{FuriganaText, Text, TextSize, TypographyVariant};
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
        <div class="intro-step" data-testid=test_id_val>
            <div class="language-bar" data-testid="intro-step-language-bar">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {t!(i18n, profile.interface_language)}
                </Text>
                <LanguageSelector selected_language=selected_language />
            </div>

            <div class="text-center mb-8">
                <Text size=TextSize::Large variant=TypographyVariant::Primary test_id="intro-step-title">
                    {t!(i18n, onboarding.intro.title)}
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted test_id="intro-step-subtitle">
                        {t!(i18n, onboarding.intro.subtitle)}
                    </Text>
                </div>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id="intro-step-hint">
                        {t!(i18n, onboarding.intro.hint)}
                    </Text>
                </div>
            </div>

            <div class="feature-showcase" data-testid="intro-step-feature-showcase">
                <div class="feature-item" data-testid="intro-step-feature-vocabulary">
                    <div class="feature-kanji">
                        <FuriganaText text="語彙".to_string() known_kanji=Default::default() />
                    </div>
                    <span class="feature-label">{t!(i18n, onboarding.intro.vocabulary)}</span>
                </div>
                <div class="feature-item" data-testid="intro-step-feature-kanji">
                    <div class="feature-kanji">
                        <FuriganaText text="漢字".to_string() known_kanji=Default::default() />
                    </div>
                    <span class="feature-label">{t!(i18n, onboarding.intro.kanji)}</span>
                </div>
                <div class="feature-item" data-testid="intro-step-feature-grammar">
                    <div class="feature-kanji">
                        <FuriganaText text="文法".to_string() known_kanji=Default::default() />
                    </div>
                    <span class="feature-label">{t!(i18n, onboarding.intro.grammar)}</span>
                </div>
            </div>

            <div class="text-center mt-6">
                <Text size=TextSize::Small variant=TypographyVariant::Muted test_id="intro-step-footer">
                    {t!(i18n, onboarding.intro.footer)}
                </Text>
            </div>
        </div>
    }
}
