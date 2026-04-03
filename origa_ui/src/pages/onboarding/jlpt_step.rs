use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

use super::onboarding_state::OnboardingState;

const JLPT_UI_OPTIONS: &[(Option<JapaneseLevel>, &str, &str)] = &[
    (None, "Не знаю", "Начну с самого начала"),
    (
        Some(JapaneseLevel::N5),
        "N5",
        "Начальный уровень — базовые слова и грамматика",
    ),
    (
        Some(JapaneseLevel::N4),
        "N4",
        "Базовый уровень — разговорный японский",
    ),
    (
        Some(JapaneseLevel::N3),
        "N3",
        "Средний уровень — повседневное общение",
    ),
    (
        Some(JapaneseLevel::N2),
        "N2",
        "Продвинутый — бизнес и формальный японский",
    ),
    (
        Some(JapaneseLevel::N1),
        "N1",
        "Эксперт — свободное владение",
    ),
];

#[component]
pub fn JlptStep(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
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
                    "Выберите уровень JLPT которого вы хотите достичь"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "jlpt-step-subtitle".to_string())>
                        "Мы подберём подходящие наборы для вашего уровня"
                    </Text>
                </div>
            </div>

            <div class="space-y-3">
                <For
                    each=move || JLPT_UI_OPTIONS.iter().enumerate()
                    key=|(idx, _)| *idx
                    children=move |(_idx, (level, label, description))| {
                        let level = *level;
                        let label = *label;
                        let description = *description;
                        let is_selected = Memo::new(move |_| state.get().selected_level == level);
                        let level_for_click = level;
                        let level_code = label.to_lowercase().replace(" ", "-");

                        view! {
                            <div
                                class=move || {
                                    let base = "p-4 border cursor-pointer transition-all";
                                    if is_selected.get() {
                                        format!("{} selected", base)
                                    } else {
                                        base.to_string()
                                    }
                                }
                                style=move || {
                                    if is_selected.get() {
                                        "border: 2px solid var(--accent-olive); background: var(--bg-warm)"
                                    } else {
                                        "border: 1px solid var(--border-dark)"
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
                                        class="w-5 h-5 border relative flex-shrink-0 transition-all"
                                        style=move || {
                                            if is_selected.get() {
                                                "border: 1px solid var(--accent-olive)"
                                            } else {
                                                "border: 1px solid var(--border-dark)"
                                            }
                                        }
                                    >
                                        {move || {
                                            if is_selected.get() {
                                                view! {
                                                    <div
                                                        class="absolute"
                                                        style="inset: 3px; background: var(--fg-black)"
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
