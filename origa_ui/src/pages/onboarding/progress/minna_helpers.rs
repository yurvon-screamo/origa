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
    extras_by_level: &HashMap<JapaneseLevel, Vec<String>>,
    selected_level: Option<JapaneseLevel>,
) -> Vec<DropdownItem> {
    let progress_keys = i18n.get_keys_untracked().onboarding().progress();
    let mut items = vec![DropdownItem {
        value: "none".to_string(),
        label: progress_keys.not_studied().inner().to_string(),
    }];

    if let Some(lvl) = selected_level
        && let Some(lessons) = lessons_by_level.get(&lvl)
    {
        for lesson in lessons {
            items.push(DropdownItem {
                value: format!("lesson_{}", lesson.lesson_number),
                label: progress_keys.lesson_number().inner().to_string().replacen(
                    "{}",
                    &lesson.lesson_number.to_string(),
                    1,
                ),
            });
        }

        if extras_by_level.get(&lvl).is_some_and(|e| !e.is_empty()) {
            items.push(DropdownItem {
                value: "extra".to_string(),
                label: progress_keys.extra().inner().to_string(),
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

pub fn collect_extras_to_import(
    extras_by_level: &HashMap<JapaneseLevel, Vec<String>>,
    selected_level: JapaneseLevel,
) -> Vec<String> {
    extras_by_level
        .get(&selected_level)
        .cloned()
        .unwrap_or_default()
}

pub fn is_lesson_in_levels(
    lesson_id: &str,
    lessons_by_level: &HashMap<JapaneseLevel, Vec<MinnaLesson>>,
) -> bool {
    lessons_by_level
        .values()
        .any(|lessons| lessons.iter().any(|l| l.id == lesson_id))
}

pub fn is_extra_in_levels(
    set_id: &str,
    extras_by_level: &HashMap<JapaneseLevel, Vec<String>>,
) -> bool {
    extras_by_level
        .values()
        .any(|extras| extras.iter().any(|id| id == set_id))
}
