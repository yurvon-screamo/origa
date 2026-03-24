use std::collections::HashMap;

use crate::ui_components::{Card, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;
use origa::traits::WellKnownSetMeta;

use super::onboarding_state::OnboardingState;

#[derive(Clone, Debug, PartialEq)]
struct LessonInfo {
    id: String,
    title: String,
    lesson_number: usize,
}

fn extract_lesson_number(id: &str) -> Option<usize> {
    let parts: Vec<&str> = id.split('_').collect();
    for part in parts.iter().rev() {
        if let Ok(num) = part.parse::<usize>() {
            return Some(num);
        }
        if let Some(stripped) = part.strip_prefix('0') {
            if let Ok(num) = stripped.parse::<usize>() {
                return Some(num);
            }
        }
    }
    None
}

fn get_sets_for_app(
    available_sets: &[WellKnownSetMeta],
    app_id: &str,
    level: JapaneseLevel,
) -> Vec<LessonInfo> {
    available_sets
        .iter()
        .filter(|s| s.set_type == app_id && s.level == level)
        .filter_map(|s| {
            extract_lesson_number(&s.id).map(|num| LessonInfo {
                id: s.id.clone(),
                title: s.title_ru.clone(),
                lesson_number: num,
            })
        })
        .collect()
}

fn group_lessons_by_level(
    available_sets: &[WellKnownSetMeta],
    app_id: &str,
) -> HashMap<JapaneseLevel, Vec<LessonInfo>> {
    let mut result: HashMap<JapaneseLevel, Vec<LessonInfo>> = HashMap::new();

    for level in [
        JapaneseLevel::N5,
        JapaneseLevel::N4,
        JapaneseLevel::N3,
        JapaneseLevel::N2,
        JapaneseLevel::N1,
    ] {
        let lessons = get_sets_for_app(available_sets, app_id, level);
        if !lessons.is_empty() {
            result.insert(level, lessons);
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

#[component]
fn LessonButton(
    lesson_num: usize,
    lesson_title: String,
    is_selected: bool,
    app_id: String,
    lessons: Vec<LessonInfo>,
    on_select: Callback<(String, usize, Vec<LessonInfo>)>,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                let base = "px-3 py-1 rounded text-sm transition-all";
                if is_selected {
                    format!("{} bg-olive-500 text-white", base)
                } else {
                    format!("{} bg-gray-100 hover:bg-gray-200", base)
                }
            }
            title=lesson_title
            on:click=move |_| {
                on_select.run((app_id.clone(), lesson_num, lessons.clone()));
            }
        >
            {"Урок "}
            {lesson_num}
        </button>
    }
}

#[component]
pub fn ProgressStep() -> impl IntoView {
    let state =
        use_context::<RwSignal<OnboardingState>>().expect("OnboardingState context not found");

    let selected_apps = Memo::new(move |_| state.get().selected_apps.clone());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let apps_with_lessons: Memo<Vec<(String, HashMap<JapaneseLevel, Vec<LessonInfo>>)>> =
        Memo::new(move |_| {
            let apps: Vec<String> = selected_apps.get().into_iter().collect();
            let sets = available_sets.get();

            apps.into_iter()
                .filter_map(|app_id| {
                    let by_level = group_lessons_by_level(&sets, &app_id);
                    if by_level.is_empty() {
                        None
                    } else {
                        Some((app_id, by_level))
                    }
                })
                .collect()
        });

    let select_lesson = Callback::new(
        move |(app_id, lesson_number, lessons): (String, usize, Vec<LessonInfo>)| {
            let lessons_to_import: Vec<LessonInfo> = lessons
                .into_iter()
                .filter(|l| l.lesson_number <= lesson_number)
                .collect();

            state.update(|s| {
                let sets: Vec<WellKnownSetMeta> = s
                    .available_sets
                    .iter()
                    .filter(|set| lessons_to_import.iter().any(|l| l.id == set.id))
                    .cloned()
                    .collect();

                for set_meta in sets {
                    s.set_app_selection(&app_id, &format!("lesson_{}", lesson_number));
                    s.add_set_to_import(set_meta);
                }
            });
        },
    );

    let current_progress = Memo::new(move |_| state.get().apps_progress.clone());

    view! {
        <div class="progress-step">
            <div class="text-center mb-6">
                <Text size=TextSize::Large variant=TypographyVariant::Primary>
                    "Ваш прогресс"
                </Text>
                <div class="mt-2">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Укажите, какие уроки вы уже прошла в выбранных приложениях"
                    </Text>
                </div>
            </div>

            <Show when=move || apps_with_lessons.get().is_empty()>
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
                    each=move || apps_with_lessons.get()
                    key=|(app_id, _)| app_id.clone()
                    children=move |(app_id, lessons_by_level)| {
                        let app_id_for_btn = app_id.clone();
                        let levels_data = vec![
                            (JapaneseLevel::N5, lessons_by_level.get(&JapaneseLevel::N5).cloned()),
                            (JapaneseLevel::N4, lessons_by_level.get(&JapaneseLevel::N4).cloned()),
                            (JapaneseLevel::N3, lessons_by_level.get(&JapaneseLevel::N3).cloned()),
                            (JapaneseLevel::N2, lessons_by_level.get(&JapaneseLevel::N2).cloned()),
                            (JapaneseLevel::N1, lessons_by_level.get(&JapaneseLevel::N1).cloned()),
                        ];

                        view! {
                            <Card class=Signal::derive(|| "p-4".to_string())>
                                <Text size=TextSize::Default variant=TypographyVariant::Primary>
                                    {app_id}
                                </Text>

                                <div class="mt-4 space-y-4">
                                    {levels_data
                                        .into_iter()
                                        .filter_map(|(level, lessons)| {
                                            lessons.map(|l| (level, l))
                                        })
                                        .map(|(level, lessons)| {
                                            let level_str = level_label(level);
                                            let app_for_lessons = app_id_for_btn.clone();
                                            let lessons_clone = lessons.clone();
                                            view! {
                                                <div class="border-l-2 border-olive-300 pl-4">
                                                    <Text size=TextSize::Small variant=TypographyVariant::Primary>
                                                        {level_str}
                                                    </Text>
                                                    <div class="mt-2 flex flex-wrap gap-2">
                                                        {lessons.into_iter().map(|lesson| {
                                                            let lesson_num = lesson.lesson_number;
                                                            let lesson_title = lesson.title.clone();
                                                            let app_for_btn = app_for_lessons.clone();
                                                            let less = lessons_clone.clone();
                                                            let is_sel = current_progress.get()
                                                                .get(&app_for_btn)
                                                                .map(|p| p == &format!("lesson_{}", lesson_num))
                                                                .unwrap_or(false);
                                                            view! {
                                                                <button
                                                                    class=move || {
                                                                        let base = "px-3 py-1 rounded text-sm transition-all";
                                                                        if is_sel {
                                                                            format!("{} bg-olive-500 text-white", base)
                                                                        } else {
                                                                            format!("{} bg-gray-100 hover:bg-gray-200", base)
                                                                        }
                                                                    }
                                                                    title=lesson_title
                                                                    on:click=move |_| {
                                                                        select_lesson.run((app_for_btn.clone(), lesson_num, less.clone()));
                                                                    }
                                                                >
                                                                    {"Урок "}
                                                                    {lesson_num}
                                                                </button>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                    }
                                </div>
                            </Card>
                        }
                    }
                />
            </div>

            <div class="mt-4">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    "При выборе урока N будут импортированы все уроки с 1 по N"
                </Text>
            </div>
        </div>
    }
}
