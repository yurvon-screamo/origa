use crate::ui_components::{Card, Dropdown, DropdownItem, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

use super::super::onboarding_state::OnboardingState;
use super::types::MinnaLesson;

#[component]
pub fn MinnaProgressSelector(
    lessons_n5: Signal<Vec<MinnaLesson>>,
    lessons_n4: Signal<Vec<MinnaLesson>>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let selected_level = RwSignal::new("none".to_string());
    let selected_lesson = RwSignal::new("none".to_string());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let level_items = vec![
        DropdownItem {
            value: "none".to_string(),
            label: "Не изучал".to_string(),
        },
        DropdownItem {
            value: "N5".to_string(),
            label: "N5 (Уроки 1-25)".to_string(),
        },
        DropdownItem {
            value: "N4".to_string(),
            label: "N4 (Уроки 26-50)".to_string(),
        },
    ];

    let parsed_level = Signal::derive(move || match selected_level.get().as_str() {
        "N5" => Some(JapaneseLevel::N5),
        "N4" => Some(JapaneseLevel::N4),
        _ => None,
    });

    let lesson_items = Signal::derive(move || {
        let level = parsed_level.get();
        let mut items = vec![DropdownItem {
            value: "none".to_string(),
            label: "Не изучал".to_string(),
        }];

        if let Some(lvl) = level {
            let lessons = match lvl {
                JapaneseLevel::N5 => lessons_n5.get(),
                JapaneseLevel::N4 => lessons_n4.get(),
                _ => return items,
            };

            for lesson in lessons.iter() {
                items.push(DropdownItem {
                    value: format!("lesson_{}", lesson.lesson_number),
                    label: format!("Урок {}", lesson.lesson_number),
                });
            }
        }
        items
    });

    let import_info = Signal::derive(move || {
        let level = parsed_level.get();
        let lesson_num = selected_lesson
            .get()
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok());

        match (level, lesson_num) {
            (Some(JapaneseLevel::N5), Some(n)) => {
                Some(format!("Будут импортированы: Уроки 1-{}", n))
            },
            (Some(JapaneseLevel::N4), Some(n)) => {
                Some(format!("Будут импортированы: Уроки 1-25 + 26-{}", n))
            },
            _ => None,
        }
    });

    Effect::new(move |_| {
        let level = parsed_level.get();
        let lesson_num = selected_lesson
            .get()
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok());

        if level.is_none() || lesson_num.is_none() {
            return;
        }

        let lessons_n5_snapshot: Vec<_> = lessons_n5.get_untracked();
        let lessons_n4_snapshot: Vec<_> = lessons_n4.get_untracked();
        let sets_snapshot: Vec<_> = available_sets.get_untracked();

        if let (Some(lvl), Some(n)) = (level, lesson_num) {
            let ids_to_import: Vec<String> = match lvl {
                JapaneseLevel::N4 => {
                    let mut ids: Vec<String> =
                        lessons_n5_snapshot.iter().map(|l| l.id.clone()).collect();
                    for lesson in lessons_n4_snapshot.iter() {
                        if lesson.lesson_number <= n {
                            ids.push(lesson.id.clone());
                        }
                    }
                    ids
                },
                JapaneseLevel::N5 => lessons_n5_snapshot
                    .iter()
                    .filter(|l| l.lesson_number <= n)
                    .map(|l| l.id.clone())
                    .collect(),
                _ => vec![],
            };

            state.update(|s| {
                s.set_app_selection("MinnaNoNihongo", &format!("{:?}_{}", lvl, n));
                let all_lessons: Vec<_> = lessons_n5_snapshot
                    .iter()
                    .chain(lessons_n4_snapshot.iter())
                    .collect();
                s.sets_to_import
                    .retain(|set| !all_lessons.iter().any(|l| l.id == set.id));
                let sets_to_add: Vec<_> = sets_snapshot
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

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <div class="flex items-center gap-3 mb-2">
                <img
                    src="/public/external_icons/minnanonihongo.png"
                    class="w-12 h-12 object-contain"
                    alt="Minna no Nihongo"
                />
                <Text size=TextSize::Default variant=TypographyVariant::Primary>
                    "Minna no Nihongo"
                </Text>
            </div>

            <div class="mt-4">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    "Уровень"
                </Text>
                <div class="mt-2">
                    <Dropdown
                        options=Signal::derive(move || level_items.clone())
                        selected=selected_level
                        placeholder=Signal::derive(|| "Выберите уровень".to_string())
                        test_id=Signal::derive(|| "minna-level-dropdown".to_string())
                    />
                </div>
            </div>

            <Show when=move || parsed_level.get().is_some()>
                <div class="mt-4">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Урок"
                    </Text>
                    <div class="mt-2">
                        <Dropdown
                            options=lesson_items
                            selected=selected_lesson
                            placeholder=Signal::derive(|| "Выберите урок".to_string())
                            test_id=Signal::derive(|| "minna-lesson-dropdown".to_string())
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
        </Card>
    }
}
