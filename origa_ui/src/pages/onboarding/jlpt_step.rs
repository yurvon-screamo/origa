use crate::i18n::*;
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

use super::onboarding_state::OnboardingState;

const JLPT_UI_OPTIONS: &[Option<JapaneseLevel>] = &[
    None,
    Some(JapaneseLevel::N5),
    Some(JapaneseLevel::N4),
    Some(JapaneseLevel::N3),
    Some(JapaneseLevel::N2),
    Some(JapaneseLevel::N1),
];

fn jlpt_label(i18n: I18nContext<Locale>, level: &Option<JapaneseLevel>) -> String {
    let locale = i18n.get_locale();
    match level {
        None => td_string!(locale, onboarding.jlpt.unknown).to_string(),
        Some(JapaneseLevel::N5) => td_string!(locale, onboarding.jlpt.n5).to_string(),
        Some(JapaneseLevel::N4) => td_string!(locale, onboarding.jlpt.n4).to_string(),
        Some(JapaneseLevel::N3) => td_string!(locale, onboarding.jlpt.n3).to_string(),
        Some(JapaneseLevel::N2) => td_string!(locale, onboarding.jlpt.n2).to_string(),
        Some(JapaneseLevel::N1) => td_string!(locale, onboarding.jlpt.n1).to_string(),
    }
}

fn jlpt_description(i18n: I18nContext<Locale>, level: &Option<JapaneseLevel>) -> String {
    let locale = i18n.get_locale();
    match level {
        None => td_string!(locale, onboarding.jlpt.unknown_desc).to_string(),
        Some(JapaneseLevel::N5) => td_string!(locale, onboarding.jlpt.n5_desc).to_string(),
        Some(JapaneseLevel::N4) => td_string!(locale, onboarding.jlpt.n4_desc).to_string(),
        Some(JapaneseLevel::N3) => td_string!(locale, onboarding.jlpt.n3_desc).to_string(),
        Some(JapaneseLevel::N2) => td_string!(locale, onboarding.jlpt.n2_desc).to_string(),
        Some(JapaneseLevel::N1) => td_string!(locale, onboarding.jlpt.n1_desc).to_string(),
    }
}

#[component]
pub fn JlptStep(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let state =
        use_context::<RwSignal<OnboardingState>>().expect("OnboardingState context not found");

    let select_level = Callback::new(move |level: Option<JapaneseLevel>| {
        state.update(|s| {
            s.set_jlpt_level(level);
        });
    });

    view! {
        <div class="jlpt-step" data-testid=test_id_val>
            <div class="text-center mb-6">
                <Text size=TextSize::Large variant=TypographyVariant::Primary test_id=Signal::derive(|| "jlpt-step-title".to_string())>
                    {t!(i18n, onboarding.jlpt.title)}
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "jlpt-step-subtitle".to_string())>
                        {t!(i18n, onboarding.jlpt.subtitle)}
                    </Text>
                </div>
            </div>

            <div class="space-y-3">
                <For
                    each=move || JLPT_UI_OPTIONS.iter().enumerate()
                    key=|(idx, _)| *idx
                    children=move |(_idx, level)| {
                        let level = *level;
                        let is_selected = Memo::new(move |_| state.get().selected_level == level);
                        let level_for_click = level;
                        let label = jlpt_label(i18n, &level);
                        let description = jlpt_description(i18n, &level);
                        let level_code = label.to_lowercase().replace(" ", "-");

                        view! {
                            <div
                                class=move || {
                                    let base = "p-4 border cursor-pointer transition-all jlpt-option";
                                    if is_selected.get() {
                                        format!("{} selected", base)
                                    } else {
                                        base.to_string()
                                    }
                                }
                                data-testid=format!("jlpt-option-{}", level_code)
                                on:click=move |_| {
                                    select_level.run(level_for_click);
                                }
                            >
                                <div class="flex items-center gap-3">
                                    <div class="flex-1">
                                        <Text size=TextSize::Default variant=TypographyVariant::Primary>
                                            {label}
                                        </Text>
                                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                            {description}
                                        </Text>
                                    </div>
                                    <div
                                        class="w-5 h-5 border relative flex-shrink-0 transition-all jlpt-option-radio"
                                    >
                                        {move || {
                                            if is_selected.get() {
                                                view! {
                                                    <div
                                                        class="absolute jlpt-option-radio-check"
                                                    ></div>
                                                }.into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
