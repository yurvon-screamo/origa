use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

use super::onboarding_state::OnboardingState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum JLPTSelection {
    None,
    N5,
    N4,
    N3,
    N2,
    N1,
}

impl JLPTSelection {
    fn label(&self) -> &'static str {
        match self {
            JLPTSelection::None => "Не знаю",
            JLPTSelection::N5 => "N5",
            JLPTSelection::N4 => "N4",
            JLPTSelection::N3 => "N3",
            JLPTSelection::N2 => "N2",
            JLPTSelection::N1 => "N1",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            JLPTSelection::None => "Начну с самого начала",
            JLPTSelection::N5 => "Начальный уровень — базовые слова и грамматика",
            JLPTSelection::N4 => "Базовый уровень — разговорный японский",
            JLPTSelection::N3 => "Средний уровень — повседневное общение",
            JLPTSelection::N2 => "Продвинутый — бизнес и формальный японский",
            JLPTSelection::N1 => "Эксперт — свободное владение",
        }
    }

    fn to_japanese_level(self) -> Option<JapaneseLevel> {
        match self {
            JLPTSelection::None => None,
            JLPTSelection::N5 => Some(JapaneseLevel::N5),
            JLPTSelection::N4 => Some(JapaneseLevel::N4),
            JLPTSelection::N3 => Some(JapaneseLevel::N3),
            JLPTSelection::N2 => Some(JapaneseLevel::N2),
            JLPTSelection::N1 => Some(JapaneseLevel::N1),
        }
    }

    fn from_japanese_level(level: Option<JapaneseLevel>) -> Self {
        match level {
            None => JLPTSelection::None,
            Some(JapaneseLevel::N5) => JLPTSelection::N5,
            Some(JapaneseLevel::N4) => JLPTSelection::N4,
            Some(JapaneseLevel::N3) => JLPTSelection::N3,
            Some(JapaneseLevel::N2) => JLPTSelection::N2,
            Some(JapaneseLevel::N1) => JLPTSelection::N1,
        }
    }
}

const JLPT_OPTIONS: [JLPTSelection; 6] = [
    JLPTSelection::None,
    JLPTSelection::N5,
    JLPTSelection::N4,
    JLPTSelection::N3,
    JLPTSelection::N2,
    JLPTSelection::N1,
];

#[component]
pub fn JlptStep() -> impl IntoView {
    let state =
        use_context::<RwSignal<OnboardingState>>().expect("OnboardingState context not found");

    let selected = Memo::new(move |_| {
        let current_level = state.get().selected_level;
        JLPTSelection::from_japanese_level(current_level)
    });

    let select_level = Callback::new(move |selection: JLPTSelection| {
        state.update(|s| {
            s.set_jlpt_level(selection.to_japanese_level());
        });
    });

    view! {
        <div class="jlpt-step">
            <div class="text-center mb-6">
                <Text size=TextSize::Large variant=TypographyVariant::Primary>
                    "Выберите ваш текущий уровень JLPT"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Мы подберём подходящие наборы слов для вашего уровня"
                    </Text>
                </div>
            </div>

            <div class="space-y-3">
                <For
                    each=move || JLPT_OPTIONS
                    key=|option| *option
                    children=move |option| {
                        let is_selected = Memo::new(move |_| selected.get() == option);
                        let option_label = option.label();
                        let option_desc = option.description();
                        let opt_for_click = option;

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
                                    select_level.run(opt_for_click);
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
                                            {option_label}
                                        </Text>
                                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                            {option_desc}
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
