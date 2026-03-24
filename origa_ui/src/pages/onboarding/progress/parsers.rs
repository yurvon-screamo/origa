use std::collections::HashMap;

use origa::domain::JapaneseLevel;
use origa::traits::WellKnownSetMeta;

use super::types::{DuolingoModule, DuolingoUnit, MigiiLesson, MinnaLesson};

pub fn extract_number_from_text(
    text: &str,
    pattern_start: &str,
    pattern_end: &str,
) -> Option<usize> {
    let start_idx = text.find(pattern_start)?;
    let after_start = &text[start_idx + pattern_start.len()..];
    let end_idx = after_start.find(pattern_end).unwrap_or(after_start.len());
    let number_str = &after_start[..end_idx].trim();
    number_str.parse::<usize>().ok()
}

pub fn parse_duolingo_module_unit(title: &str, is_ru: bool) -> Option<(usize, usize)> {
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

pub fn parse_migii_level_lesson(title: &str) -> Option<(JapaneseLevel, usize)> {
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

pub fn parse_minna_lesson(title: &str) -> Option<usize> {
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

pub fn parse_duolingo_modules(
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

pub fn parse_migii_lessons(sets: &[WellKnownSetMeta]) -> HashMap<JapaneseLevel, Vec<MigiiLesson>> {
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

pub fn parse_minna_lessons(sets: &[WellKnownSetMeta], prefix: &str) -> Vec<MinnaLesson> {
    let mut lessons: Vec<MinnaLesson> = sets
        .iter()
        .filter(|s| s.id.starts_with(prefix))
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
