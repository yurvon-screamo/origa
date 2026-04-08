use crate::i18n::*;
use crate::pages::shared::DailyLoadList;
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

use super::onboarding_state::OnboardingState;

#[component]
pub fn LoadStep(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let state =
        use_context::<RwSignal<OnboardingState>>().expect("OnboardingState context not found");

    let local_load = RwSignal::new(state.get_untracked().daily_load);

    Effect::new(move |_| {
        let current = local_load.get();
        state.update(|s| {
            if s.daily_load != current {
                s.set_daily_load(current);
            }
        });
    });

    view! {
        <div class="load-step" data-testid=test_id_val>
            <div class="text-center mb-8">
                <Text size=TextSize::Large variant=TypographyVariant::Primary test_id=Signal::derive(|| "load-step-title".to_string())>
                    {t!(i18n, onboarding.load.title)}
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted test_id=Signal::derive(|| "load-step-subtitle".to_string())>
                        {t!(i18n, onboarding.load.subtitle)}
                    </Text>
                </div>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "load-step-hint".to_string())>
                        {t!(i18n, onboarding.load.hint)}
                    </Text>
                </div>
            </div>

            <DailyLoadList selected_load=local_load />

            <div class="text-center mt-6">
                <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "load-step-footer".to_string())>
                    {t!(i18n, onboarding.load.footer)}
                </Text>
            </div>
        </div>
    }
}
