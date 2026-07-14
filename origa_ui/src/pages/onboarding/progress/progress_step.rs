use crate::i18n::*;
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

use super::super::onboarding_state::OnboardingState;
use super::app_type::{AppType, parse_app_type};
use super::duolingo_selector::DuolingoProgressSelector;
use super::irodori_selector::IrodoriProgressSelector;
use super::migii_selector::MigiiProgressSelector;
use super::minna_selector::MinnaProgressSelector;
use super::set_parsers::{
    parse_duolingo_modules, parse_irodori_lessons, parse_migii_lessons,
    parse_minna_lessons_by_level,
};

#[component]
pub fn ProgressStep(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let state =
        use_context::<RwSignal<OnboardingState>>().expect("OnboardingState context not found");

    let selected_apps = Memo::new(move |_| state.get().selected_apps.clone());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let app_list = Memo::new(move |_| {
        let mut v: Vec<_> = selected_apps.get().into_iter().collect();
        v.sort();
        v
    });

    view! {
        <div class="progress-step" data-testid=test_id_val>
            <div class="text-center mb-6">
                <Text size=TextSize::Large variant=TypographyVariant::Primary test_id=Signal::derive(|| "progress-step-title".to_string())>
                    {t!(i18n, onboarding.progress.title)}
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "progress-step-subtitle".to_string())>
                        {t!(i18n, onboarding.progress.subtitle)}
                    </Text>
                </div>
            </div>

            <Show when=move || app_list.get().is_empty()>
                <div class="text-center py-8">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted test_id=Signal::derive(|| "progress-step-empty".to_string())>
                        {t!(i18n, onboarding.progress.empty)}
                    </Text>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "progress-step-empty-hint".to_string())>
                            {t!(i18n, onboarding.progress.empty_hint)}
                        </Text>
                    </div>
                </div>
            </Show>

            <div class="space-y-4">
                <For
                    each=move || app_list.get()
                    key=|app_id| app_id.clone()
                    children=move |app_id| {
                        let app_type = parse_app_type(&app_id);
                        let sets = available_sets;

                        match app_type {
                            Some(AppType::DuolingoRu) => {
                                let modules_signal = Signal::derive(move || {
                                    parse_duolingo_modules(&sets.get(), "DuolingoRu", true)
                                });
                                view! {
                                    <DuolingoProgressSelector
                                        app_id=app_id.clone()
                                        is_ru=true
                                        modules=modules_signal
                                        state=state
                                    />
                                }.into_any()
                            }
                            Some(AppType::DuolingoEn) => {
                                let modules_signal = Signal::derive(move || {
                                    parse_duolingo_modules(&sets.get(), "DuolingoEn", false)
                                });
                                view! {
                                    <DuolingoProgressSelector
                                        app_id=app_id.clone()
                                        is_ru=false
                                        modules=modules_signal
                                        state=state
                                    />
                                }.into_any()
                            }
                            Some(AppType::Migii) => {
                                let lessons_signal = Signal::derive(move || {
                                    parse_migii_lessons(&sets.get())
                                });
                                view! {
                                    <MigiiProgressSelector
                                        lessons_by_level=lessons_signal
                                        state=state
                                    />
                                }.into_any()
                            }
                            Some(AppType::MinnaNoNihongo) => {
                                let lessons_by_level_signal = Signal::derive(move || {
                                    parse_minna_lessons_by_level(&sets.get())
                                });
                                view! {
                                    <MinnaProgressSelector
                                        lessons_by_level=lessons_by_level_signal
                                        state=state
                                    />
                                }.into_any()
                            }
                            Some(AppType::Irodori) => {
                                let lessons_nyuumon_signal = Signal::derive(move || {
                                    parse_irodori_lessons(&sets.get(), "irodori_nyuumon_")
                                });
                                let lessons_shokyuu1_signal = Signal::derive(move || {
                                    parse_irodori_lessons(&sets.get(), "irodori_shokyuu1_")
                                });
                                let lessons_shokyuu2_signal = Signal::derive(move || {
                                    parse_irodori_lessons(&sets.get(), "irodori_shokyuu2_")
                                });
                                view! {
                                    <IrodoriProgressSelector
                                        lessons_nyuumon=lessons_nyuumon_signal
                                        lessons_shokyuu1=lessons_shokyuu1_signal
                                        lessons_shokyuu2=lessons_shokyuu2_signal
                                        state=state
                                    />
                                }.into_any()
                            }
                            None => ().into_any(),
                        }
                    }
                />
            </div>
        </div>
    }
}
