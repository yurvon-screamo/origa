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
                    "Выберите ваш текущий уровень JLPT"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "jlpt-step-subtitle".to_string())>
                        "Мы подберём подходящие наборы слов для вашего уровня"
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

                        view! {
                            <div
                                class=move || {
                                    let base = "p-4 rounded-lg border-2 cursor-pointer transition-all";
                                    if is_selected.get() {
                                        format!("{} border-olive-500 bg-olive-50", base)
                                    } else {
                                        format!("{} border-gray-200 hover:border-gray-300", base)
                                    }
                                }
                                on:click=move |_| {
                                    select_level.run(level_for_click);
                                }
                            >
                                <div class="flex items-center gap-3">
                                    <div class=move || {
                                        let dot_class = "w-4 h-4 rounded-full border-2";
                                        if is_selected.get() {
                                            format!("{} border-olive-500 bg-olive-500", dot_class)
                                        } else {
                                            format!("{} border-gray-300", dot_class)
                                        }
                                    }/>
                                    <div class="flex-1">
                                        <Text size=TextSize::Default variant=TypographyVariant::Primary>
                                            {label}
                                        </Text>
                                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                            {description}
                                        </Text>
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
