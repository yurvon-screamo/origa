use crate::i18n::{I18nContext, Locale};
use crate::ui_components::DropdownItem;

use super::types::IrodoriLesson;

pub fn build_book_items(i18n: &I18nContext<Locale>) -> Vec<DropdownItem> {
    let not_studied = i18n
        .get_keys_untracked()
        .onboarding()
        .progress()
        .not_studied()
        .inner()
        .to_string();

    vec![
        DropdownItem {
            value: "none".to_string(),
            label: not_studied,
        },
        DropdownItem {
            value: "nyuumon".to_string(),
            label: "入門 (N5)".to_string(),
        },
        DropdownItem {
            value: "shokyuu1".to_string(),
            label: "初級1 (N4)".to_string(),
        },
        DropdownItem {
            value: "shokyuu2".to_string(),
            label: "初級2 (N4)".to_string(),
        },
    ]
}

pub fn build_lesson_items(
    i18n: &I18nContext<Locale>,
    nyuumon: &[IrodoriLesson],
    shokyuu1: &[IrodoriLesson],
    shokyuu2: &[IrodoriLesson],
    book: &str,
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

    let lessons = match book {
        "nyuumon" => nyuumon,
        "shokyuu1" => shokyuu1,
        "shokyuu2" => shokyuu2,
        _ => return items,
    };

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

    items
}

pub fn collect_lessons_to_import(
    nyuumon: &[IrodoriLesson],
    shokyuu1: &[IrodoriLesson],
    shokyuu2: &[IrodoriLesson],
    book: &str,
    lesson_n: usize,
) -> Vec<String> {
    match book {
        "nyuumon" => nyuumon
            .iter()
            .filter(|l| l.lesson_number <= lesson_n)
            .map(|l| l.id.clone())
            .collect(),
        "shokyuu1" => {
            let mut ids: Vec<String> = nyuumon.iter().map(|l| l.id.clone()).collect();
            for lesson in shokyuu1.iter() {
                if lesson.lesson_number <= lesson_n {
                    ids.push(lesson.id.clone());
                }
            }
            ids
        },
        "shokyuu2" => {
            let mut ids: Vec<String> = nyuumon.iter().map(|l| l.id.clone()).collect();
            ids.extend(shokyuu1.iter().map(|l| l.id.clone()));
            for lesson in shokyuu2.iter() {
                if lesson.lesson_number <= lesson_n {
                    ids.push(lesson.id.clone());
                }
            }
            ids
        },
        _ => vec![],
    }
}

pub fn is_irodori_lesson(id: &str) -> bool {
    id.starts_with("irodori_nyuumon_")
        || id.starts_with("irodori_shokyuu1_")
        || id.starts_with("irodori_shokyuu2_")
}
