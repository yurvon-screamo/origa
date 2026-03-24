use std::collections::HashMap;

use crate::ui_components::{Card, Dropdown, DropdownItem, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;
use origa::traits::WellKnownSetMeta;

use super::onboarding_state::OnboardingState;

#[derive(Clone, Debug)]
struct DuolingoModule {
    module_number: usize,
    units: Vec<DuolingoUnit>,
}

#[derive(Clone, Debug)]
struct DuolingoUnit {
    id: String,
    unit_number: usize,
}

#[derive(Clone, Debug)]
struct MigiiLesson {
    id: String,
    lesson_number: usize,
}

#[derive(Clone, Debug)]
struct MinnaLesson {
    id: String,
    lesson_number: usize,
}

fn extract_number_from_text(text: &str, pattern_start: &str, pattern_end: &str) -> Option<usize> {
    let start_idx = text.find(pattern_start)?;
    let after_start = &text[start_idx + pattern_start.len()..];
    let end_idx = after_start.find(pattern_end).unwrap_or(after_start.len());
    let number_str = &after_start[..end_idx].trim();
    number_str.parse::<usize>().ok()
}

fn parse_duolingo_module_unit(title: &str, is_ru: bool) -> Option<(usize, usize)> {
    if is_ru {
        let module_num = extract_number_from_text(title, "Модуль ", " Раздел")?;
        let unit_num = extract_number_from_text(title, "Раздел ", "")?;
        Some((module_num, unit_num))
    } else {
        let module_num = extract_number_from_text(title, "Section ", " Unit")?;
        let unit_num = extract_number_from_text(title, "Unit ", "")?;
        Some((module_num, unit_num))
    }
}

fn parse_migii_level_lesson(title: &str) -> Option<(JapaneseLevel, usize)> {
    let level = if title.contains("N5") {
        JapaneseLevel::N5
    } else if title.contains("N4") {
        JapaneseLevel::N4
    } else if title.contains("N3") {
        JapaneseLevel::N3
    } else if title.contains("N2") {
        JapaneseLevel::N2
    } else if title.contains("N1") {
        JapaneseLevel::N1
    } else {
        return None;
    };

    let lesson_num = extract_number_from_text(title, "Урок ", "")
        .or_else(|| extract_number_from_text(title, "Lesson ", ""))?;

    Some((level, lesson_num))
}

fn parse_minna_lesson(title: &str) -> Option<usize> {
    extract_number_from_text(title, "Урок ", "")
        .or_else(|| extract_number_from_text(title, "Lesson ", ""))
        .or_else(|| {
            let re_patterns = ["minna_n5_", "minna_n4_"];
            for pattern in re_patterns {
                if title.contains(pattern) {
                    let start = title.find(pattern)? + pattern.len();
                    let rest = &title[start..];
                    let end = rest.find('_').unwrap_or(rest.len());
                    let num_str = &rest[..end];
                    return num_str.parse::<usize>().ok();
                }
            }
            None
        })
}

fn parse_duolingo_modules(
    sets: &[WellKnownSetMeta],
    app_id: &str,
    is_ru: bool,
) -> Vec<DuolingoModule> {
    let mut modules_map: HashMap<usize, Vec<DuolingoUnit>> = HashMap::new();

    for set in sets.iter().filter(|s| s.set_type == app_id) {
        if let Some((module_num, unit_num)) = parse_duolingo_module_unit(&set.title_ru, is_ru)
            .or_else(|| parse_duolingo_module_unit(&set.title_en, false))
        {
            let unit = DuolingoUnit {
                id: set.id.clone(),
                unit_number: unit_num,
            };
            modules_map.entry(module_num).or_default().push(unit);
        }
    }

    let mut modules: Vec<DuolingoModule> = modules_map
        .into_iter()
        .map(|(module_number, mut units)| {
            units.sort_by_key(|u| u.unit_number);
            DuolingoModule {
                module_number,
                units,
            }
        })
        .collect();
    modules.sort_by_key(|m| m.module_number);
    modules
}

fn parse_migii_lessons(sets: &[WellKnownSetMeta]) -> HashMap<JapaneseLevel, Vec<MigiiLesson>> {
    let mut by_level: HashMap<JapaneseLevel, Vec<MigiiLesson>> = HashMap::new();

    for set in sets.iter().filter(|s| s.set_type == "Migii") {
        if let Some((level, lesson_num)) = parse_migii_level_lesson(&set.title_ru)
            .or_else(|| parse_migii_level_lesson(&set.title_en))
        {
            let lesson = MigiiLesson {
                id: set.id.clone(),
                lesson_number: lesson_num,
            };
            by_level.entry(level).or_default().push(lesson);
        }
    }

    for lessons in by_level.values_mut() {
        lessons.sort_by_key(|l| l.lesson_number);
    }

    by_level
}

fn parse_minna_lessons(sets: &[WellKnownSetMeta]) -> Vec<MinnaLesson> {
    let mut lessons: Vec<MinnaLesson> = sets
        .iter()
        .filter(|s| s.set_type == "MinnaNoNihongo")
        .filter_map(|set| {
            parse_minna_lesson(&set.title_ru)
                .or_else(|| parse_minna_lesson(&set.title_en))
                .or_else(|| parse_minna_lesson(&set.id))
                .map(|lesson_number| MinnaLesson {
                    id: set.id.clone(),
                    lesson_number,
                })
        })
        .collect();

    lessons.sort_by_key(|l| l.lesson_number);
    lessons
}

#[component]
fn DuolingoProgressSelector(
    app_id: String,
    is_ru: bool,
    modules: Vec<DuolingoModule>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let selected_module = RwSignal::new(None::<usize>);
    let selected_unit = RwSignal::new(None::<usize>);
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let module_items: Vec<DropdownItem> = {
        let mut items = vec![DropdownItem {
            value: "none".to_string(),
            label: "Не изучал".to_string(),
        }];
        for module in &modules {
            items.push(DropdownItem {
                value: format!("module_{}", module.module_number),
                label: format!("Модуль {}", module.module_number),
            });
        }
        items
    };

    let modules_for_unit_items = modules.clone();
    let unit_items = Signal::derive(move || {
        let module_num = selected_module.get();
        let mut items = vec![DropdownItem {
            value: "none".to_string(),
            label: "Не изучал".to_string(),
        }];

        if let Some(num) = module_num
            && let Some(module) = modules_for_unit_items
                .iter()
                .find(|m| m.module_number == num)
        {
            for unit in &module.units {
                items.push(DropdownItem {
                    value: format!("unit_{}", unit.unit_number),
                    label: format!("Раздел {}", unit.unit_number),
                });
            }
        }
        items
    });

    let import_info = Signal::derive(move || {
        let module_num = selected_module.get();
        let unit_num = selected_unit.get();

        match (module_num, unit_num) {
            (Some(m), Some(u)) => {
                Some(format!("Будут импортированы: Модуль {} Разделы 1-{}", m, u))
            }
            (Some(m), None) => Some(format!("Выберите раздел в Модуле {}", m)),
            _ => None,
        }
    });

    let modules_for_effect = modules;
    let app_id_for_effect = app_id;
    Effect::new(move |_| {
        let module_num = selected_module.get();
        let unit_num = selected_unit.get();
        let aid = app_id_for_effect.clone();
        let mods = modules_for_effect.clone();
        let sets = available_sets.get();

        if let (Some(m), Some(u)) = (module_num, unit_num)
            && let Some(module) = mods.iter().find(|mod_| mod_.module_number == m)
        {
            let units_to_import: Vec<String> = module
                .units
                .iter()
                .filter(|unit| unit.unit_number <= u)
                .map(|unit| unit.id.clone())
                .collect();

            state.update(|s| {
                s.set_app_selection(&aid, &format!("module_{}_unit_{}", m, u));
                s.sets_to_import.retain(|set| {
                    !mods
                        .iter()
                        .any(|mod_| mod_.units.iter().any(|u_| u_.id == set.id))
                });
                let sets_to_add: Vec<_> = sets
                    .iter()
                    .filter(|set_meta| units_to_import.contains(&set_meta.id))
                    .cloned()
                    .collect();
                for set_meta in sets_to_add {
                    s.add_set_to_import(set_meta);
                }
            });
        }
    });

    let selected_module_value = RwSignal::new(
        selected_module
            .get()
            .map(|n| format!("module_{}", n))
            .unwrap_or_else(|| "none".to_string()),
    );
    let selected_unit_value = RwSignal::new(
        selected_unit
            .get()
            .map(|n| format!("unit_{}", n))
            .unwrap_or_else(|| "none".to_string()),
    );

    Effect::new(move |_| {
        let val = selected_module_value.get();
        selected_module.set(
            val.strip_prefix("module_")
                .and_then(|s| s.parse::<usize>().ok()),
        );
    });

    Effect::new(move |_| {
        let val = selected_unit_value.get();
        selected_unit.set(
            val.strip_prefix("unit_")
                .and_then(|s| s.parse::<usize>().ok()),
        );
    });

    let app_label = if is_ru {
        "Duolingo (RU)"
    } else {
        "Duolingo (EN)"
    };

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <Text size=TextSize::Default variant=TypographyVariant::Primary>
                {app_label}
            </Text>

            <div class="mt-4 space-y-4">
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Модуль"
                    </Text>
                    <div class="mt-2">
                        <Dropdown
                            _options=Signal::derive(move || module_items.clone())
                            _selected=selected_module_value
                            _placeholder=Signal::derive(|| "Выберите модуль".to_string())
                        />
                    </div>
                </div>

                <Show when=move || selected_module.get().is_some()>
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Раздел"
                        </Text>
                        <div class="mt-2">
                            <Dropdown
                                _options=unit_items
                                _selected=selected_unit_value
                                _placeholder=Signal::derive(|| "Выберите раздел".to_string())
                            />
                        </div>
                    </div>
                </Show>

                <Show when=move || import_info.get().is_some()>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {move || import_info.get().unwrap_or_default()}
                        </Text>
                    </div>
                </Show>
            </div>
        </Card>
    }
}

#[component]
fn MigiiProgressSelector(
    lessons_by_level: HashMap<JapaneseLevel, Vec<MigiiLesson>>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let selected_level = RwSignal::new(None::<JapaneseLevel>);
    let selected_lesson = RwSignal::new(None::<usize>);
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let level_items = vec![
        DropdownItem {
            value: "none".to_string(),
            label: "Не изучал".to_string(),
        },
        DropdownItem {
            value: "N5".to_string(),
            label: "N5".to_string(),
        },
        DropdownItem {
            value: "N4".to_string(),
            label: "N4".to_string(),
        },
        DropdownItem {
            value: "N3".to_string(),
            label: "N3".to_string(),
        },
        DropdownItem {
            value: "N2".to_string(),
            label: "N2".to_string(),
        },
        DropdownItem {
            value: "N1".to_string(),
            label: "N1".to_string(),
        },
    ];

    let lessons_by_level_for_items = lessons_by_level.clone();
    let lesson_items = Signal::derive(move || {
        let level = selected_level.get();
        let mut items = vec![DropdownItem {
            value: "none".to_string(),
            label: "Не изучал".to_string(),
        }];

        if let Some(lvl) = level
            && let Some(lessons) = lessons_by_level_for_items.get(&lvl)
        {
            for lesson in lessons {
                items.push(DropdownItem {
                    value: format!("lesson_{}", lesson.lesson_number),
                    label: format!("Урок {}", lesson.lesson_number),
                });
            }
        }
        items
    });

    let import_info = Signal::derive(move || {
        let level = selected_level.get();
        let lesson = selected_lesson.get();

        match (level, lesson) {
            (Some(lvl), Some(n)) => Some(format!(
                "Будут импортированы: {} Уроки 1-{}",
                level_to_str(lvl),
                n
            )),
            (Some(lvl), None) => Some(format!("Выберите урок для {}", level_to_str(lvl))),
            _ => None,
        }
    });

    let lessons_by_level_for_effect = lessons_by_level.clone();
    Effect::new(move |_| {
        let level = selected_level.get();
        let lesson_num = selected_lesson.get();
        let lessons_by = lessons_by_level_for_effect.clone();
        let sets = available_sets.get();

        if let (Some(lvl), Some(lesson_n)) = (level, lesson_num)
            && let Some(lessons) = lessons_by.get(&lvl)
        {
            let ids_to_import: Vec<String> = lessons
                .iter()
                .filter(|l| l.lesson_number <= lesson_n)
                .map(|l| l.id.clone())
                .collect();

            state.update(|s| {
                s.set_app_selection("Migii", &format!("{:?}_{}", lvl, lesson_n));
                s.sets_to_import.retain(|set| {
                    !lessons_by
                        .values()
                        .any(|lessons| lessons.iter().any(|l| l.id == set.id))
                });
                let sets_to_add: Vec<_> = sets
                    .iter()
                    .filter(|set_meta| ids_to_import.contains(&set_meta.id))
                    .cloned()
                    .collect();
                for set_meta in sets_to_add {
                    s.add_set_to_import(set_meta);
                }
            });
        }
    });

    let selected_level_value = RwSignal::new(
        selected_level
            .get()
            .map(|l| level_to_str(l).to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    let selected_lesson_value = RwSignal::new(
        selected_lesson
            .get()
            .map(|n| format!("lesson_{}", n))
            .unwrap_or_else(|| "none".to_string()),
    );

    Effect::new(move |_| {
        let val = selected_level_value.get();
        selected_level.set(match val.as_str() {
            "N5" => Some(JapaneseLevel::N5),
            "N4" => Some(JapaneseLevel::N4),
            "N3" => Some(JapaneseLevel::N3),
            "N2" => Some(JapaneseLevel::N2),
            "N1" => Some(JapaneseLevel::N1),
            _ => None,
        });
    });

    Effect::new(move |_| {
        let val = selected_lesson_value.get();
        selected_lesson.set(
            val.strip_prefix("lesson_")
                .and_then(|s| s.parse::<usize>().ok()),
        );
        selected_lesson.set(selected_lesson.get());
    });

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <Text size=TextSize::Default variant=TypographyVariant::Primary>
                "Migii"
            </Text>

            <div class="mt-4 space-y-4">
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Уровень"
                    </Text>
                    <div class="mt-2">
                        <Dropdown
                            _options=Signal::derive(move || level_items.clone())
                            _selected=selected_level_value
                            _placeholder=Signal::derive(|| "Выберите уровень".to_string())
                        />
                    </div>
                </div>

                <Show when=move || selected_level.get().is_some()>
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Урок"
                        </Text>
                        <div class="mt-2">
                            <Dropdown
                                _options=lesson_items
                                _selected=selected_lesson_value
                                _placeholder=Signal::derive(|| "Выберите урок".to_string())
                            />
                        </div>
                    </div>
                </Show>

                <Show when=move || import_info.get().is_some()>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {move || import_info.get().unwrap_or_default()}
                        </Text>
                    </div>
                </Show>
            </div>
        </Card>
    }
}

#[component]
fn MinnaProgressSelector(
    lessons: Vec<MinnaLesson>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let selected_lesson = RwSignal::new(None::<usize>);
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let lesson_items = {
        let mut items = vec![DropdownItem {
            value: "none".to_string(),
            label: "Не изучал".to_string(),
        }];
        for lesson in &lessons {
            items.push(DropdownItem {
                value: format!("lesson_{}", lesson.lesson_number),
                label: format!("Урок {}", lesson.lesson_number),
            });
        }
        items
    };

    let import_info = Signal::derive(move || {
        selected_lesson
            .get()
            .map(|n| format!("Будут импортированы: Уроки 1-{}", n))
    });

    let lessons_for_effect = lessons.clone();
    Effect::new(move |_| {
        let lesson_num = selected_lesson.get();
        let lessons_ref = lessons_for_effect.clone();
        let sets = available_sets.get();

        if let Some(n) = lesson_num {
            let ids_to_import: Vec<String> = lessons_ref
                .iter()
                .filter(|l| l.lesson_number <= n)
                .map(|l| l.id.clone())
                .collect();

            state.update(|s| {
                s.set_app_selection("MinnaNoNihongo", &format!("lesson_{}", n));
                s.sets_to_import
                    .retain(|set| !lessons_ref.iter().any(|l| l.id == set.id));
                let sets_to_add: Vec<_> = sets
                    .iter()
                    .filter(|set_meta| ids_to_import.contains(&set_meta.id))
                    .cloned()
                    .collect();
                for set_meta in sets_to_add {
                    s.add_set_to_import(set_meta);
                }
            });
        }
    });

    let selected_lesson_value = RwSignal::new(
        selected_lesson
            .get()
            .map(|n| format!("lesson_{}", n))
            .unwrap_or_else(|| "none".to_string()),
    );

    Effect::new(move |_| {
        let val = selected_lesson_value.get();
        selected_lesson.set(
            val.strip_prefix("lesson_")
                .and_then(|s| s.parse::<usize>().ok()),
        );
    });

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <Text size=TextSize::Default variant=TypographyVariant::Primary>
                "MinnaNoNihongo"
            </Text>

            <div class="mt-4">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    "Урок"
                </Text>
                <div class="mt-2">
                    <Dropdown
                        _options=Signal::derive(move || lesson_items.clone())
                        _selected=selected_lesson_value
                        _placeholder=Signal::derive(|| "Выберите урок".to_string())
                    />
                </div>
            </div>

            <Show when=move || import_info.get().is_some()>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {move || import_info.get().unwrap_or_default()}
                    </Text>
                </div>
            </Show>
        </Card>
    }
}

fn level_to_str(level: JapaneseLevel) -> &'static str {
    match level {
        JapaneseLevel::N5 => "N5",
        JapaneseLevel::N4 => "N4",
        JapaneseLevel::N3 => "N3",
        JapaneseLevel::N2 => "N2",
        JapaneseLevel::N1 => "N1",
    }
}

#[derive(Clone)]
enum AppType {
    DuolingoRu,
    DuolingoEn,
    Migii,
    MinnaNoNihongo,
}

fn parse_app_type(app_id: &str) -> Option<AppType> {
    match app_id {
        "DuolingoRu" => Some(AppType::DuolingoRu),
        "DuolingoEn" => Some(AppType::DuolingoEn),
        "Migii" => Some(AppType::Migii),
        "MinnaNoNihongo" => Some(AppType::MinnaNoNihongo),
        _ => None,
    }
}

#[component]
pub fn ProgressStep() -> impl IntoView {
    let state =
        use_context::<RwSignal<OnboardingState>>().expect("OnboardingState context not found");

    let selected_apps = Memo::new(move |_| state.get().selected_apps.clone());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let app_list = Memo::new(move |_| selected_apps.get().into_iter().collect::<Vec<_>>());

    view! {
        <div class="progress-step">
            <div class="text-center mb-6">
                <Text size=TextSize::Large variant=TypographyVariant::Primary>
                    "Ваш прогресс"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Выберите пройденные разделы в каждом приложении"
                    </Text>
                </div>
            </div>

            <Show when=move || app_list.get().is_empty()>
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

            <div class="space-y-4">
                <For
                    each=move || app_list.get()
                    key=|app_id| app_id.clone()
                    children=move |app_id| {
                        let app_type = parse_app_type(&app_id);
                        let sets = available_sets.get();

                        match app_type {
                            Some(AppType::DuolingoRu) => {
                                let modules = parse_duolingo_modules(&sets, "DuolingoRu", true);
                                view! {
                                    <DuolingoProgressSelector
                                        app_id=app_id.clone()
                                        is_ru=true
                                        modules=modules
                                        state=state
                                    />
                                }.into_any()
                            }
                            Some(AppType::DuolingoEn) => {
                                let modules = parse_duolingo_modules(&sets, "DuolingoEn", false);
                                view! {
                                    <DuolingoProgressSelector
                                        app_id=app_id.clone()
                                        is_ru=false
                                        modules=modules
                                        state=state
                                    />
                                }.into_any()
                            }
                            Some(AppType::Migii) => {
                                let lessons = parse_migii_lessons(&sets);
                                view! {
                                    <MigiiProgressSelector
                                        lessons_by_level=lessons
                                        state=state
                                    />
                                }.into_any()
                            }
                            Some(AppType::MinnaNoNihongo) => {
                                let lessons = parse_minna_lessons(&sets);
                                view! {
                                    <MinnaProgressSelector
                                        lessons=lessons
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
