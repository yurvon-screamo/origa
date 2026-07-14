use std::collections::HashMap;

use crate::i18n::{I18nContext, Locale};
use crate::ui_components::DropdownItem;
use origa::domain::JapaneseLevel;

use super::app_type::level_to_str;
use super::types::MinnaLesson;

pub fn build_level_items(
    i18n: &I18nContext<Locale>,
    lessons_by_level: &HashMap<JapaneseLevel, Vec<MinnaLesson>>,
) -> Vec<DropdownItem> {
    let not_studied = i18n
        .get_keys_untracked()
        .onboarding()
        .progress()
        .not_studied()
        .inner()
        .to_string();

    let mut items = vec![DropdownItem {
        value: "none".to_string(),
        label: not_studied,
    }];

    for &level in JapaneseLevel::ALL.iter() {
        if lessons_by_level.contains_key(&level) {
            let label = level_to_str(level);
            items.push(DropdownItem {
                value: label.to_string(),
                label: label.to_string(),
            });
        }
    }

    items
}

pub fn build_lesson_items(
    i18n: &I18nContext<Locale>,
    lessons_by_level: &HashMap<JapaneseLevel, Vec<MinnaLesson>>,
    selected_level: Option<JapaneseLevel>,
) -> Vec<DropdownItem> {
    let mut items = vec![DropdownItem {
        value: "none".to_string(),
        label: i18n
            .get_keys_untracked()
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
                    .get_keys_untracked()
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

pub fn collect_lessons_to_import(
    lessons_by_level: &HashMap<JapaneseLevel, Vec<MinnaLesson>>,
    selected_level: JapaneseLevel,
    lesson_limit: usize,
) -> Vec<String> {
    let mut ids = Vec::new();

    for &level in JapaneseLevel::ALL.iter() {
        let Some(lessons) = lessons_by_level.get(&level) else {
            continue;
        };

        if level == selected_level {
            for lesson in lessons {
                if lesson.lesson_number <= lesson_limit {
                    ids.push(lesson.id.clone());
                }
            }
        } else if level < selected_level {
            for lesson in lessons {
                ids.push(lesson.id.clone());
            }
        }
    }

    ids
}

pub fn is_lesson_in_levels(
    lesson_id: &str,
    lessons_by_level: &HashMap<JapaneseLevel, Vec<MinnaLesson>>,
) -> bool {
    lessons_by_level
        .values()
        .any(|lessons| lessons.iter().any(|l| l.id == lesson_id))
}
