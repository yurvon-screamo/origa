use crate::i18n::*;
use crate::ui_components::{NativeLanguageToggle, Text, TextSize, TypographyVariant};
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
                <div class="flex items-center w-full relative" data-testid="intro-step-language-bar">
                    <Text size=TextSize::Large variant=TypographyVariant::Primary test_id="intro-step-title" class="w-full text-center">
                        {t!(i18n, onboarding.intro.title)}
                    </Text>
                    <div class="absolute right-0">
                        <NativeLanguageToggle selected_language=selected_language test_id=Signal::derive(|| "intro-lang-toggle".to_string()) />
                    </div>
                </div>
                <Text size=TextSize::Default variant=TypographyVariant::Muted test_id="intro-step-subtitle">
                    {t!(i18n, onboarding.intro.subtitle)}
                </Text>
            </div>
        </div>
    }
}
