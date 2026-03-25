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
    // Try Russian format first if is_ru
    if is_ru {
        // Try "Модуль X Раздел Y" format
        if let Some(module_num) = extract_number_from_text(title, "Модуль ", " Раздел")
        {
            if let Some(unit_num) = extract_number_from_text(title, "Раздел ", "") {
                return Some((module_num, unit_num));
            }
        }

        // Try "Модуль X Юнит Y" format (alternative Russian)
        if let Some(module_num) = extract_number_from_text(title, "Модуль ", " Юнит") {
            if let Some(unit_num) = extract_number_from_text(title, "Юнит ", "") {
                return Some((module_num, unit_num));
            }
        }

        // Try extracting from ID if title parsing fails
        // e.g., "duolingo_ru_module_1_unit_2"
        if let Some(module_num) = extract_number_from_text(title, "module_", "_unit") {
            if let Some(unit_num) = extract_number_from_text(title, "unit_", "") {
                return Some((module_num, unit_num));
            }
        }
    } else {
        // Try English format "Section X Unit Y"
        if let Some(module_num) = extract_number_from_text(title, "Section ", " Unit") {
            if let Some(unit_num) = extract_number_from_text(title, "Unit ", "") {
                return Some((module_num, unit_num));
            }
        }

        // Try "Module X Unit Y" format (alternative English)
        if let Some(module_num) = extract_number_from_text(title, "Module ", " Unit") {
            if let Some(unit_num) = extract_number_from_text(title, "Unit ", "") {
                return Some((module_num, unit_num));
            }
        }

        // Try extracting from ID if title parsing fails
        if let Some(module_num) = extract_number_from_text(title, "module_", "_unit") {
            if let Some(unit_num) = extract_number_from_text(title, "unit_", "") {
                return Some((module_num, unit_num));
            }
        }
    }

    // Log warning if parsing fails
    tracing::warn!("Failed to parse Duolingo module/unit from title: {}", title);
    None
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
    // Try Russian format "Урок X"
    if let Some(num) = extract_number_from_text(title, "Урок ", "") {
        return Some(num);
    }

    // Try English format "Lesson X"
    if let Some(num) = extract_number_from_text(title, "Lesson ", "") {
        return Some(num);
    }

    // Try ID format "minna_n5_X" or "minna_n4_X"
    let re_patterns = ["minna_n5_", "minna_n4_"];
    for pattern in re_patterns {
        if title.contains(pattern) {
            if let Some(start_idx) = title.find(pattern) {
                let start = start_idx + pattern.len();
                let rest = &title[start..];
                let end = rest.find('_').unwrap_or(rest.len());
                let num_str = &rest[..end];
                if let Ok(num) = num_str.parse::<usize>() {
                    return Some(num);
                }
            }
        }
    }

    // Log warning if parsing fails
    tracing::warn!("Failed to parse Minna lesson number from title: {}", title);
    None
}

pub fn parse_duolingo_modules(
    sets: &[WellKnownSetMeta],
    app_id: &str,
    is_ru: bool,
) -> Vec<DuolingoModule> {
    let mut modules_map: HashMap<usize, Vec<DuolingoUnit>> = HashMap::new();
    let mut parsed_count = 0;
    let mut total_count = 0;

    for set in sets.iter().filter(|s| s.set_type == app_id) {
        total_count += 1;

        // Try parsing from title_ru first, then title_en, then id
        let parsed = parse_duolingo_module_unit(&set.title_ru, is_ru)
            .or_else(|| parse_duolingo_module_unit(&set.title_en, false))
            .or_else(|| parse_duolingo_module_unit(&set.id, is_ru));

        if let Some((module_num, unit_num)) = parsed {
            parsed_count += 1;
            let unit = DuolingoUnit {
                id: set.id.clone(),
                unit_number: unit_num,
            };
            modules_map.entry(module_num).or_default().push(unit);
        }
    }

    // Log parsing statistics
    if total_count > 0 {
        tracing::info!(
            "Duolingo {} parser: {}/{} sets parsed successfully",
            app_id,
            parsed_count,
            total_count
        );

        if parsed_count == 0 {
            tracing::warn!(
                "No Duolingo {} sets could be parsed! Check title format in data. \
                 Example titles: {:?}",
                app_id,
                sets.iter()
                    .filter(|s| s.set_type == app_id)
                    .take(3)
                    .map(|s| (&s.title_ru, &s.title_en, &s.id))
                    .collect::<Vec<_>>()
            );
        }
    } else {
        tracing::warn!("No Duolingo {} sets found in available_sets", app_id);
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
    let mut parsed_count = 0;
    let mut total_count = 0;

    for set in sets.iter().filter(|s| s.set_type == "Migii") {
        total_count += 1;

        // Try parsing from title_ru first, then title_en
        let parsed = parse_migii_level_lesson(&set.title_ru)
            .or_else(|| parse_migii_level_lesson(&set.title_en));

        if let Some((level, lesson_num)) = parsed {
            parsed_count += 1;
            let lesson = MigiiLesson {
                id: set.id.clone(),
                lesson_number: lesson_num,
            };
            by_level.entry(level).or_default().push(lesson);
        }
    }

    // Log parsing statistics
    if total_count > 0 {
        tracing::info!(
            "Migii parser: {}/{} sets parsed successfully",
            parsed_count,
            total_count
        );

        if parsed_count == 0 {
            tracing::warn!(
                "No Migii sets could be parsed! Check title format in data. \
                 Example titles: {:?}",
                sets.iter()
                    .filter(|s| s.set_type == "Migii")
                    .take(3)
                    .map(|s| (&s.title_ru, &s.title_en, &s.id))
                    .collect::<Vec<_>>()
            );
        }
    } else {
        tracing::warn!("No Migii sets found in available_sets");
    }

    for lessons in by_level.values_mut() {
        lessons.sort_by_key(|l| l.lesson_number);
    }

    by_level
}

pub fn parse_minna_lessons(sets: &[WellKnownSetMeta], prefix: &str) -> Vec<MinnaLesson> {
    let mut parsed_count = 0;
    let mut total_count = 0;

    let mut lessons: Vec<MinnaLesson> = sets
        .iter()
        .filter(|s| s.id.starts_with(prefix))
        .filter_map(|set| {
            total_count += 1;

            // Try parsing from title_ru, title_en, then id
            let parsed = parse_minna_lesson(&set.title_ru)
                .or_else(|| parse_minna_lesson(&set.title_en))
                .or_else(|| parse_minna_lesson(&set.id));

            if parsed.is_some() {
                parsed_count += 1;
            }

            parsed.map(|lesson_number| MinnaLesson {
                id: set.id.clone(),
                lesson_number,
            })
        })
        .collect();

    // Log parsing statistics
    if total_count > 0 {
        tracing::info!(
            "Minna {} parser: {}/{} sets parsed successfully",
            prefix,
            parsed_count,
            total_count
        );

        if parsed_count == 0 {
            tracing::warn!(
                "No Minna {} sets could be parsed! Check title format in data. \
                 Example titles: {:?}",
                prefix,
                sets.iter()
                    .filter(|s| s.id.starts_with(prefix))
                    .take(3)
                    .map(|s| (&s.title_ru, &s.title_en, &s.id))
                    .collect::<Vec<_>>()
            );
        }
    } else {
        tracing::warn!("No Minna {} sets found in available_sets", prefix);
    }

    lessons.sort_by_key(|l| l.lesson_number);
    lessons
}
