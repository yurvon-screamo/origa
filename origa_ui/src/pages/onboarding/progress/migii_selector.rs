use std::collections::HashMap;

use crate::ui_components::{Card, Dropdown, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

use super::super::onboarding_state::OnboardingState;
use super::app_type::level_to_str;
use super::migii_helpers::{
    build_lesson_items, build_level_items, collect_lessons_to_import, is_lesson_in_levels,
};
use super::types::MigiiLesson;

#[component]
pub fn MigiiProgressSelector(
    lessons_by_level: HashMap<JapaneseLevel, Vec<MigiiLesson>>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let selected_level_value = RwSignal::new("none".to_string());
    let selected_lesson_value = RwSignal::new("none".to_string());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let level_items = build_level_items();

    let lessons_by_level_for_items = lessons_by_level.clone();
    let lesson_items = Signal::derive(move || {
        let level_str = selected_level_value.get();
        let level = match level_str.as_str() {
            "N5" => Some(JapaneseLevel::N5),
            "N4" => Some(JapaneseLevel::N4),
            "N3" => Some(JapaneseLevel::N3),
            "N2" => Some(JapaneseLevel::N2),
            "N1" => Some(JapaneseLevel::N1),
            _ => None,
        };
        build_lesson_items(&lessons_by_level_for_items, level)
    });

    let import_info = Signal::derive(move || {
        let level_str = selected_level_value.get();
        let lesson_str = selected_lesson_value.get();

        let level = match level_str.as_str() {
            "N5" => Some(JapaneseLevel::N5),
            "N4" => Some(JapaneseLevel::N4),
            "N3" => Some(JapaneseLevel::N3),
            "N2" => Some(JapaneseLevel::N2),
            "N1" => Some(JapaneseLevel::N1),
            _ => None,
        };

        let lesson_num = lesson_str
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok());

        match (level, lesson_num) {
            (Some(lvl), Some(n)) => Some(format!(
                "Будут импортированы: {} Уроки 1-{}",
                level_to_str(lvl),
                n
            )),
            (Some(lvl), None) => Some(format!("Выберите урок для {}", level_to_str(lvl))),
            _ => None,
        }
    });

    // Single Effect that handles both level and lesson selection
    Effect::new(move |_| {
        let level_str = selected_level_value.get();
        let lesson_str = selected_lesson_value.get();

        let level = match level_str.as_str() {
            "N5" => Some(JapaneseLevel::N5),
            "N4" => Some(JapaneseLevel::N4),
            "N3" => Some(JapaneseLevel::N3),
            "N2" => Some(JapaneseLevel::N2),
            "N1" => Some(JapaneseLevel::N1),
            _ => None,
        };

        let lesson_num = lesson_str
            .strip_prefix("lesson_")
            .and_then(|s| s.parse::<usize>().ok());

        if let (Some(lvl), Some(lesson_n)) = (level, lesson_num)
            && let Some(lessons) = lessons_by_level.get(&lvl)
        {
            // Collect IDs to import - use iterator without cloning the whole map
            let ids_to_import = collect_lessons_to_import(lessons, lesson_n);

            let lessons_by_ref = lessons_by_level.clone();
            let sets = available_sets.get();

            state.update(|s| {
                s.set_app_selection("Migii", &format!("{:?}_{}", lvl, lesson_n));
                // Remove old Migii sets
                s.sets_to_import
                    .retain(|set| !is_lesson_in_levels(set.id.as_str(), &lessons_by_ref));
                // Add new sets
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
                            options=Signal::derive(move || level_items.clone())
                            selected=selected_level_value
                            placeholder=Signal::derive(|| "Выберите уровень".to_string())
                            test_id=Signal::derive(|| "migii-level-dropdown".to_string())
                        />
                    </div>
                </div>

                <Show when=move || selected_level_value.get() != "none">
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Урок"
                        </Text>
                        <div class="mt-2">
                            <Dropdown
                                options=lesson_items
                                selected=selected_lesson_value
                                placeholder=Signal::derive(|| "Выберите урок".to_string())
                                test_id=Signal::derive(|| "migii-lesson-dropdown".to_string())
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
