use std::collections::HashMap;

use crate::i18n::{I18nContext, Locale};
use crate::ui_components::DropdownItem;
use origa::domain::JapaneseLevel;

use super::types::MigiiLesson;

pub fn build_level_items(i18n: &I18nContext<Locale>) -> Vec<DropdownItem> {
    vec![
        DropdownItem {
            value: "none".to_string(),
            label: i18n
                .get_keys()
                .onboarding()
                .progress()
                .not_studied()
                .inner()
                .to_string(),
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
    i18n: &I18nContext<Locale>,
    lessons_by_level: &HashMap<JapaneseLevel, Vec<MigiiLesson>>,
    selected_level: Option<JapaneseLevel>,
) -> Vec<DropdownItem> {
    let mut items = vec![DropdownItem {
        value: "none".to_string(),
        label: i18n
            .get_keys()
            .onboarding()
            .progress()
            .not_studied()
            .inner()
            .to_string(),
    }];

    if let Some(lvl) = selected_level
        && let Some(lessons) = lessons_by_level.get(&lvl)
    {
        for lesson in lessons {
            items.push(DropdownItem {
                value: format!("lesson_{}", lesson.lesson_number),
                label: i18n
                    .get_keys()
                    .onboarding()
                    .progress()
                    .lesson_number()
                    .inner()
                    .to_string()
                    .replacen("{}", &lesson.lesson_number.to_string(), 1),
            });
        }
    }
    items
}

pub fn collect_lessons_to_import_all_levels(
    lessons_by_level: &HashMap<JapaneseLevel, Vec<MigiiLesson>>,
    selected_level: JapaneseLevel,
    lesson_limit: usize,
) -> Vec<String> {
    let mut ids = Vec::new();
    let levels_order = [
        JapaneseLevel::N5,
        JapaneseLevel::N4,
        JapaneseLevel::N3,
        JapaneseLevel::N2,
        JapaneseLevel::N1,
    ];

    for &level in &levels_order {
        if let Some(lessons) = lessons_by_level.get(&level) {
            if level == selected_level {
                for lesson in lessons {
                    if lesson.lesson_number <= lesson_limit {
                        ids.push(lesson.id.clone());
                    }
                }
            } else if is_lower_level(level, selected_level) {
                for lesson in lessons {
                    ids.push(lesson.id.clone());
                }
            }
        }
    }

    ids
}

fn is_lower_level(level: JapaneseLevel, selected: JapaneseLevel) -> bool {
    matches!(
        (level, selected),
        (JapaneseLevel::N5, JapaneseLevel::N4)
            | (JapaneseLevel::N5, JapaneseLevel::N3)
            | (JapaneseLevel::N5, JapaneseLevel::N2)
            | (JapaneseLevel::N5, JapaneseLevel::N1)
            | (JapaneseLevel::N4, JapaneseLevel::N3)
            | (JapaneseLevel::N4, JapaneseLevel::N2)
            | (JapaneseLevel::N4, JapaneseLevel::N1)
            | (JapaneseLevel::N3, JapaneseLevel::N2)
            | (JapaneseLevel::N3, JapaneseLevel::N1)
            | (JapaneseLevel::N2, JapaneseLevel::N1)
    )
}

pub fn is_lesson_in_levels(
    lesson_id: &str,
    lessons_by_level: &HashMap<JapaneseLevel, Vec<MigiiLesson>>,
) -> bool {
    lessons_by_level
        .values()
        .any(|lessons| lessons.iter().any(|l| l.id == lesson_id))
}
