use crate::i18n::{I18nContext, Locale};
use origa::domain::JapaneseLevel;

use super::app_type::level_to_str;

pub fn build_cumulative_import_info(
    i18n: &I18nContext<Locale>,
    level: Option<JapaneseLevel>,
    lesson_num: Option<usize>,
) -> Option<String> {
    let progress_keys = i18n.get_keys().onboarding().progress();

    let template = match (level, lesson_num) {
        (Some(JapaneseLevel::N5), Some(n)) => progress_keys
            .import_n5_lessons()
            .inner()
            .to_string()
            .replacen("{}", &n.to_string(), 1),
        (Some(JapaneseLevel::N4), Some(n)) => progress_keys
            .import_n5_all_n4_lessons()
            .inner()
            .to_string()
            .replacen("{}", &n.to_string(), 1),
        (Some(JapaneseLevel::N3), Some(n)) => progress_keys
            .import_n5_n4_all_n3_lessons()
            .inner()
            .to_string()
            .replacen("{}", &n.to_string(), 1),
        (Some(JapaneseLevel::N2), Some(n)) => progress_keys
            .import_n5_n4_n3_all_n2_lessons()
            .inner()
            .to_string()
            .replacen("{}", &n.to_string(), 1),
        (Some(JapaneseLevel::N1), Some(n)) => progress_keys
            .import_n5_n4_n3_n2_all_n1_lessons()
            .inner()
            .to_string()
            .replacen("{}", &n.to_string(), 1),
        (Some(lvl), None) => progress_keys
            .select_lesson_for()
            .inner()
            .to_string()
            .replacen("{}", level_to_str(lvl), 1),
        _ => return None,
    };

    Some(template)
}
