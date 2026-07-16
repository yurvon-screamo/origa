use std::collections::HashMap;

use origa::domain::JapaneseLevel;
use origa::domain::WellKnownSetMeta;

use super::parsers::{
    minna_level_from_id, parse_duolingo_module_unit, parse_irodori_lesson,
    parse_migii_level_lesson, parse_minna_lesson,
};
use super::types::{DuolingoModule, DuolingoUnit, IrodoriLesson, MigiiLesson, MinnaLesson};

pub(super) fn parse_duolingo_modules(
    sets: &[WellKnownSetMeta],
    app_id: &str,
    is_ru: bool,
) -> Vec<DuolingoModule> {
    let mut modules_map: HashMap<usize, Vec<DuolingoUnit>> = HashMap::new();
    let mut parsed_count = 0;
    let mut total_count = 0;

    for set in sets.iter().filter(|s| s.set_type == app_id) {
        total_count += 1;

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

pub(super) fn parse_migii_lessons(
    sets: &[WellKnownSetMeta],
) -> HashMap<JapaneseLevel, Vec<MigiiLesson>> {
    let mut by_level: HashMap<JapaneseLevel, Vec<MigiiLesson>> = HashMap::new();
    let mut parsed_count = 0;
    let mut total_count = 0;

    for set in sets.iter().filter(|s| s.set_type == "Migii") {
        total_count += 1;

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

pub(super) fn parse_minna_lessons_by_level(
    sets: &[WellKnownSetMeta],
) -> HashMap<JapaneseLevel, Vec<MinnaLesson>> {
    let mut by_level: HashMap<JapaneseLevel, Vec<MinnaLesson>> = HashMap::new();
    let mut parsed_count = 0;
    let mut total_count = 0;

    for set in sets.iter().filter(|s| s.set_type == "MinnaNoNihongo") {
        total_count += 1;

        let Some(level) = minna_level_from_id(&set.id) else {
            continue;
        };

        let parsed = parse_minna_lesson(&set.title_ru)
            .or_else(|| parse_minna_lesson(&set.title_en))
            .or_else(|| parse_minna_lesson(&set.id));

        if let Some(lesson_number) = parsed {
            parsed_count += 1;
            by_level.entry(level).or_default().push(MinnaLesson {
                id: set.id.clone(),
                lesson_number,
            });
        }
    }

    if total_count > 0 {
        tracing::info!(
            "Minna parser: {}/{} sets parsed successfully",
            parsed_count,
            total_count
        );

        if parsed_count == 0 {
            tracing::warn!(
                "No Minna sets could be parsed! Check title format in data. \
                 Example titles: {:?}",
                sets.iter()
                    .filter(|s| s.set_type == "MinnaNoNihongo")
                    .take(3)
                    .map(|s| (&s.title_ru, &s.title_en, &s.id))
                    .collect::<Vec<_>>()
            );
        }
    } else {
        tracing::warn!("No MinnaNoNihongo sets found in available_sets");
    }

    for lessons in by_level.values_mut() {
        lessons.sort_by_key(|l| l.lesson_number);
    }

    by_level
}

pub(super) fn parse_minna_extras_by_level(
    sets: &[WellKnownSetMeta],
) -> HashMap<JapaneseLevel, Vec<String>> {
    let mut by_level: HashMap<JapaneseLevel, Vec<String>> = HashMap::new();

    for set in sets.iter().filter(|s| s.set_type == "MinnaNoNihongo") {
        let Some(level) = minna_level_from_id(&set.id) else {
            continue;
        };

        let is_lesson = parse_minna_lesson(&set.title_ru)
            .or_else(|| parse_minna_lesson(&set.title_en))
            .or_else(|| parse_minna_lesson(&set.id))
            .is_some();

        if !is_lesson {
            by_level.entry(level).or_default().push(set.id.clone());
        }
    }

    for extras in by_level.values_mut() {
        extras.sort();
    }

    by_level
}

pub(super) fn parse_irodori_lessons(sets: &[WellKnownSetMeta], prefix: &str) -> Vec<IrodoriLesson> {
    let mut parsed_count = 0;
    let mut total_count = 0;

    let mut lessons: Vec<IrodoriLesson> = sets
        .iter()
        .filter(|s| s.id.starts_with(prefix))
        .filter_map(|set| {
            total_count += 1;

            let parsed = parse_irodori_lesson(&set.title_ru)
                .or_else(|| parse_irodori_lesson(&set.title_en))
                .or_else(|| parse_irodori_lesson(&set.id));

            if parsed.is_some() {
                parsed_count += 1;
            }

            parsed.map(|lesson_number| IrodoriLesson {
                id: set.id.clone(),
                lesson_number,
            })
        })
        .collect();

    if total_count > 0 {
        tracing::info!(
            "Irodori {} parser: {}/{} sets parsed successfully",
            prefix,
            parsed_count,
            total_count
        );

        if parsed_count == 0 {
            tracing::warn!(
                "No Irodori {} sets could be parsed! Check title format in data. \
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
        tracing::warn!("No Irodori {} sets found in available_sets", prefix);
    }

    lessons.sort_by_key(|l| l.lesson_number);
    lessons
}

#[cfg(test)]
mod set_parser_tests {
    use super::*;

    #[test]
    fn test_parse_duolingo_modules_single_module_single_unit() {
        let sets = vec![WellKnownSetMeta {
            id: "duolingo_ru_1".to_string(),
            set_type: "DuolingoRu".to_string(),
            level: JapaneseLevel::N5,
            title_ru: "Duolingo 「RU」 - Модуль 5 Раздел 1".to_string(),
            title_en: "Duolingo 「EN」 - Section 5 Unit 1".to_string(),
            desc_ru: String::new(),
            desc_en: String::new(),
            word_count: 0,
        }];

        let modules = parse_duolingo_modules(&sets, "DuolingoRu", true);

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].module_number, 5);
        assert_eq!(modules[0].units.len(), 1);
        assert_eq!(modules[0].units[0].unit_number, 1);
        assert_eq!(modules[0].units[0].id, "duolingo_ru_1");
    }

    #[test]
    fn test_parse_duolingo_modules_multiple_units_same_module() {
        let sets = vec![
            WellKnownSetMeta {
                id: "duolingo_ru_1".to_string(),
                set_type: "DuolingoRu".to_string(),
                level: JapaneseLevel::N5,
                title_ru: "Duolingo 「RU」 - Модуль 5 Раздел 1".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "duolingo_ru_2".to_string(),
                set_type: "DuolingoRu".to_string(),
                level: JapaneseLevel::N5,
                title_ru: "Duolingo 「RU」 - Модуль 5 Раздел 3".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
        ];

        let modules = parse_duolingo_modules(&sets, "DuolingoRu", true);

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].module_number, 5);
        assert_eq!(modules[0].units.len(), 2);
        assert_eq!(modules[0].units[0].unit_number, 1);
        assert_eq!(modules[0].units[1].unit_number, 3);
    }

    #[test]
    fn test_parse_duolingo_modules_filters_by_app_id() {
        let sets = vec![
            WellKnownSetMeta {
                id: "duolingo_ru_1".to_string(),
                set_type: "DuolingoRu".to_string(),
                level: JapaneseLevel::N5,
                title_ru: "Duolingo 「RU」 - Модуль 5 Раздел 1".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "duolingo_en_1".to_string(),
                set_type: "DuolingoEn".to_string(),
                level: JapaneseLevel::N5,
                title_ru: String::new(),
                title_en: "Duolingo 「EN」 - Section 3 Unit 2".to_string(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
        ];

        let modules_ru = parse_duolingo_modules(&sets, "DuolingoRu", true);
        let modules_en = parse_duolingo_modules(&sets, "DuolingoEn", false);

        assert_eq!(modules_ru.len(), 1);
        assert_eq!(modules_ru[0].module_number, 5);

        assert_eq!(modules_en.len(), 1);
        assert_eq!(modules_en[0].module_number, 3);
    }

    #[test]
    fn test_parse_migii_lessons_groups_by_level() {
        let sets = vec![
            WellKnownSetMeta {
                id: "migii_n5_1".to_string(),
                set_type: "Migii".to_string(),
                level: JapaneseLevel::N5,
                title_ru: "Migii N5 Урок 1".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "migii_n5_2".to_string(),
                set_type: "Migii".to_string(),
                level: JapaneseLevel::N5,
                title_ru: "Migii N5 Lesson 5".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "migii_n4_1".to_string(),
                set_type: "Migii".to_string(),
                level: JapaneseLevel::N4,
                title_ru: "Migii N4 Урок 10".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
        ];

        let lessons = parse_migii_lessons(&sets);

        assert_eq!(lessons.len(), 2);
        assert_eq!(lessons.get(&JapaneseLevel::N5).unwrap().len(), 2);
        assert_eq!(lessons.get(&JapaneseLevel::N4).unwrap().len(), 1);
        assert_eq!(lessons.get(&JapaneseLevel::N5).unwrap()[0].lesson_number, 1);
        assert_eq!(lessons.get(&JapaneseLevel::N5).unwrap()[1].lesson_number, 5);
    }

    #[test]
    fn test_parse_minna_lessons_by_level_groups_and_filters() {
        let sets = vec![
            WellKnownSetMeta {
                id: "minna_n5_1".to_string(),
                set_type: "MinnaNoNihongo".to_string(),
                level: JapaneseLevel::N5,
                title_ru: "Minna no Nihongo N5 Урок 1".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "minna_n5_5".to_string(),
                set_type: "MinnaNoNihongo".to_string(),
                level: JapaneseLevel::N5,
                title_ru: String::new(),
                title_en: "Minna no Nihongo N5 Lesson 5".to_string(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "minna_n4_1".to_string(),
                set_type: "MinnaNoNihongo".to_string(),
                level: JapaneseLevel::N4,
                title_ru: "Minna no Nihongo N4 Урок 1".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "minna_n3_7".to_string(),
                set_type: "MinnaNoNihongo".to_string(),
                level: JapaneseLevel::N3,
                title_ru: String::new(),
                title_en: "Minna no Nihongo N3 Lesson 7".to_string(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "migii_n5_1".to_string(),
                set_type: "Migii".to_string(),
                level: JapaneseLevel::N5,
                title_ru: "Migii N5 Урок 1".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
        ];

        let by_level = parse_minna_lessons_by_level(&sets);

        assert_eq!(by_level.len(), 3);
        let n5 = by_level.get(&JapaneseLevel::N5).unwrap();
        assert_eq!(n5.len(), 2);
        assert_eq!(n5[0].lesson_number, 1);
        assert_eq!(n5[1].lesson_number, 5);
        assert_eq!(by_level.get(&JapaneseLevel::N4).unwrap().len(), 1);
        let n3 = by_level.get(&JapaneseLevel::N3).unwrap();
        assert_eq!(n3.len(), 1);
        assert_eq!(n3[0].lesson_number, 7);
        assert!(!by_level.contains_key(&JapaneseLevel::N2));
    }

    #[test]
    fn test_parse_minna_extras_by_level_separates_non_lesson_sets() {
        let sets = vec![
            WellKnownSetMeta {
                id: "minna_n2_13".to_string(),
                set_type: "MinnaNoNihongo".to_string(),
                level: JapaneseLevel::N2,
                title_ru: "Minna no Nihongo N2 Урок 13".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "minna_n2_extra".to_string(),
                set_type: "MinnaNoNihongo".to_string(),
                level: JapaneseLevel::N2,
                title_ru: "Minna no Nihongo N2 Дополнительно".to_string(),
                title_en: "Minna no Nihongo N2 Extra".to_string(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 158,
            },
            WellKnownSetMeta {
                id: "minna_n3_extra".to_string(),
                set_type: "MinnaNoNihongo".to_string(),
                level: JapaneseLevel::N3,
                title_ru: "Minna no Nihongo N3 Дополнительно".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
        ];

        let extras = parse_minna_extras_by_level(&sets);

        assert_eq!(extras.len(), 2);
        let n2 = extras.get(&JapaneseLevel::N2).unwrap();
        assert_eq!(n2, &vec!["minna_n2_extra".to_string()]);
        assert_eq!(extras.get(&JapaneseLevel::N3).unwrap().len(), 1);
        // regular lesson (minna_n2_13) must NOT appear in extras
        assert!(!n2.contains(&"minna_n2_13".to_string()));
    }

    #[test]
    fn test_parse_irodori_lessons_filters_by_prefix() {
        let sets = vec![
            WellKnownSetMeta {
                id: "irodori_nyuumon_01".to_string(),
                set_type: "Irodori".to_string(),
                level: JapaneseLevel::N5,
                title_ru: "Irodori 入門 Урок 1".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "irodori_nyuumon_05".to_string(),
                set_type: "Irodori".to_string(),
                level: JapaneseLevel::N5,
                title_ru: String::new(),
                title_en: "Irodori Nyuumon Lesson 5".to_string(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
            WellKnownSetMeta {
                id: "irodori_shokyuu1_01".to_string(),
                set_type: "Irodori".to_string(),
                level: JapaneseLevel::N4,
                title_ru: "Irodori 初級1 Урок 1".to_string(),
                title_en: String::new(),
                desc_ru: String::new(),
                desc_en: String::new(),
                word_count: 0,
            },
        ];

        let nyuumon_lessons = parse_irodori_lessons(&sets, "irodori_nyuumon_");
        let shokyuu1_lessons = parse_irodori_lessons(&sets, "irodori_shokyuu1_");

        assert_eq!(nyuumon_lessons.len(), 2);
        assert_eq!(shokyuu1_lessons.len(), 1);
        assert_eq!(nyuumon_lessons[0].lesson_number, 1);
        assert_eq!(nyuumon_lessons[1].lesson_number, 5);
    }
}
