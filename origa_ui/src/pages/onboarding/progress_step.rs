use std::collections::HashMap;

use crate::ui_components::{Card, Dropdown, DropdownItem, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;
use origa::traits::WellKnownSetMeta;

use super::onboarding_state::OnboardingState;

type AppModulesByLevel = Vec<(String, HashMap<JapaneseLevel, Vec<ModuleInfo>>)>;

#[derive(Clone, Debug, PartialEq)]
struct ModuleInfo {
    id: String,
    title: String,
    module_number: usize,
}

fn extract_module_number(id: &str) -> Option<usize> {
    let parts: Vec<&str> = id.split('_').collect();
    for part in parts.iter().rev() {
        if let Ok(num) = part.parse::<usize>() {
            return Some(num);
        }
        if let Some(stripped) = part.strip_prefix('0')
            && let Ok(num) = stripped.parse::<usize>()
        {
            return Some(num);
        }
    }
    None
}

fn natural_sort_modules(a: &ModuleInfo, b: &ModuleInfo) -> std::cmp::Ordering {
    a.module_number.cmp(&b.module_number)
}

fn get_sets_for_app(
    available_sets: &[WellKnownSetMeta],
    app_id: &str,
    level: JapaneseLevel,
) -> Vec<ModuleInfo> {
    let mut modules: Vec<ModuleInfo> = available_sets
        .iter()
        .filter(|s| s.set_type == app_id && s.level == level)
        .filter_map(|s| {
            extract_module_number(&s.id).map(|num| ModuleInfo {
                id: s.id.clone(),
                title: s.title_ru.clone(),
                module_number: num,
            })
        })
        .collect();

    modules.sort_by(natural_sort_modules);
    modules
}

fn group_modules_by_level(
    available_sets: &[WellKnownSetMeta],
    app_id: &str,
) -> HashMap<JapaneseLevel, Vec<ModuleInfo>> {
    let mut result: HashMap<JapaneseLevel, Vec<ModuleInfo>> = HashMap::new();

    for level in [
        JapaneseLevel::N5,
        JapaneseLevel::N4,
        JapaneseLevel::N3,
        JapaneseLevel::N2,
        JapaneseLevel::N1,
    ] {
        let modules = get_sets_for_app(available_sets, app_id, level);
        if !modules.is_empty() {
            result.insert(level, modules);
        }
    }

    result
}

fn level_label(level: JapaneseLevel) -> &'static str {
    match level {
        JapaneseLevel::N5 => "N5",
        JapaneseLevel::N4 => "N4",
        JapaneseLevel::N3 => "N3",
        JapaneseLevel::N2 => "N2",
        JapaneseLevel::N1 => "N1",
    }
}

fn create_dropdown_items(modules: &[ModuleInfo]) -> Vec<DropdownItem> {
    let mut items: Vec<DropdownItem> = modules
        .iter()
        .map(|m| DropdownItem {
            value: format!("module_{}", m.module_number),
            label: format!("Модуль {}", m.module_number),
        })
        .collect();
    items.insert(
        0,
        DropdownItem {
            value: "none".to_string(),
            label: "Не изучал".to_string(),
        },
    );
    items
}

#[component]
fn AppLevelSelect(
    app_id: String,
    level: JapaneseLevel,
    modules: Vec<ModuleInfo>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let level_str = level_label(level);
    let dropdown_items = create_dropdown_items(&modules);

    let initial_value = state
        .get()
        .apps_progress
        .get(&app_id)
        .cloned()
        .unwrap_or_else(|| "none".to_string());
    let selected = RwSignal::new(initial_value);
    let app_id_signal = RwSignal::new(app_id);
    let modules_signal = RwSignal::new(modules);

    Effect::new(move |_| {
        let value = selected.get();
        let aid = app_id_signal.get();
        let mods = modules_signal.get();

        if value != "none"
            && let Some(module_num_str) = value.strip_prefix("module_")
            && let Ok(module_num) = module_num_str.parse::<usize>()
        {
            let modules_to_import: Vec<String> = mods
                .iter()
                .filter(|m| m.module_number <= module_num)
                .map(|m| m.id.clone())
                .collect();

            state.update(|s| {
                s.set_app_selection(&aid, &value);
                let sets_to_add: Vec<_> = s
                    .available_sets
                    .iter()
                    .filter(|set_meta| modules_to_import.contains(&set_meta.id))
                    .cloned()
                    .collect();
                for set_meta in sets_to_add {
                    s.add_set_to_import(set_meta);
                }
            });
        }
    });

    view! {
        <div>
            <Text size=TextSize::Small variant=TypographyVariant::Primary>
                {"Уровень "}
                {level_str}
            </Text>
            <div class="mt-2">
                <Dropdown
                    _options=Signal::derive(move || dropdown_items.clone())
                    _selected=selected
                    _placeholder=Signal::derive(|| "Выберите модуль".to_string())
                />
            </div>
        </div>
    }
}

#[component]
pub fn ProgressStep() -> impl IntoView {
    let state =
        use_context::<RwSignal<OnboardingState>>().expect("OnboardingState context not found");

    let selected_apps = Memo::new(move |_| state.get().selected_apps.clone());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let apps_with_modules: Memo<AppModulesByLevel> = Memo::new(move |_| {
        let apps: Vec<String> = selected_apps.get().into_iter().collect();
        let sets = available_sets.get();

        apps.into_iter()
            .filter_map(|app_id| {
                let by_level = group_modules_by_level(&sets, &app_id);
                if by_level.is_empty() {
                    None
                } else {
                    Some((app_id, by_level))
                }
            })
            .collect()
    });

    view! {
        <div class="progress-step">
            <div class="text-center mb-6">
                <Text size=TextSize::Large variant=TypographyVariant::Primary>
                    "Ваш прогресс"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Выберите пройденные модули в каждом приложении"
                    </Text>
                </div>
            </div>

            <Show when=move || apps_with_modules.get().is_empty()>
                <div class="text-center py-8">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        "Вы не выбрали ни одно приложение"
                    </Text>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Вернитесь на шаг назад, чтобы выбрать приложения"
                        </Text>
                    </div>
                </div>
            </Show>

            <div class="space-y-6">
                <For
                    each=move || apps_with_modules.get()
                    key=|(app_id, _)| app_id.clone()
                    children=move |(app_id, modules_by_level)| {
                        let app_id_for_card = app_id.clone();
                        let level_entries: Vec<(JapaneseLevel, Vec<ModuleInfo>)> = [
                            JapaneseLevel::N5,
                            JapaneseLevel::N4,
                            JapaneseLevel::N3,
                            JapaneseLevel::N2,
                            JapaneseLevel::N1,
                        ]
                        .iter()
                        .filter_map(|&level| {
                            modules_by_level.get(&level).map(|m| (level, m.clone()))
                        })
                        .collect();

                        view! {
                            <Card class=Signal::derive(|| "p-4".to_string())>
                                <Text size=TextSize::Default variant=TypographyVariant::Primary>
                                    {app_id}
                                </Text>

                                <div class="mt-4 space-y-4">
                                    <For
                                        each=move || level_entries.clone()
                                        key=|(level, _)| *level
                                        children=move |(level, modules)| {
                                            view! {
                                                <AppLevelSelect
                                                    app_id=app_id_for_card.clone()
                                                    level=level
                                                    modules=modules
                                                    state=state
                                                />
                                            }
                                        }
                                    />
                                </div>

                                <div class="mt-4">
                                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                        "При выборе модуля N будут импортированы все модули с 1 по N"
                                    </Text>
                                </div>
                            </Card>
                        }
                    }
                />
            </div>
        </div>
    }
}