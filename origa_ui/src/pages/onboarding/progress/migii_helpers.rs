use std::collections::HashMap;

use crate::ui_components::DropdownItem;
use origa::domain::JapaneseLevel;

use super::types::MigiiLesson;

pub fn build_level_items() -> Vec<DropdownItem> {
    vec![
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
    ]
}

pub fn build_lesson_items(
    lessons_by_level: &HashMap<JapaneseLevel, Vec<MigiiLesson>>,
    selected_level: Option<JapaneseLevel>,
) -> Vec<DropdownItem> {
    let mut items = vec![DropdownItem {
        value: "none".to_string(),
        label: "Не изучал".to_string(),
    }];

    if let Some(lvl) = selected_level
        && let Some(lessons) = lessons_by_level.get(&lvl)
    {
        for lesson in lessons {
            items.push(DropdownItem {
                value: format!("lesson_{}", lesson.lesson_number),
                label: format!("Урок {}", lesson.lesson_number),
            });
        }
    }
    items
}

pub fn collect_lessons_to_import(lessons: &[MigiiLesson], lesson_limit: usize) -> Vec<String> {
    lessons
        .iter()
        .filter(|l| l.lesson_number <= lesson_limit)
        .map(|l| l.id.clone())
        .collect()
}

pub fn is_lesson_in_levels(
    lesson_id: &str,
    lessons_by_level: &HashMap<JapaneseLevel, Vec<MigiiLesson>>,
) -> bool {
    lessons_by_level
        .values()
        .any(|lessons| lessons.iter().any(|l| l.id == lesson_id))
}
