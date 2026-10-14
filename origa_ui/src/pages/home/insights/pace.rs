//! Строка «Темп»: среднее повторений за день против прошлых 14 дней.

use crate::i18n::{I18nContext, Locale, td_string};
use origa::domain::insights::PaceInsight;

/// Ветка фразы темпа; чистая функция для прямого тестирования выбора.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PacePhrase {
    Empty,
    Up,
    Down,
    Flat,
}

pub(crate) fn pace_phrase_kind(pace: Option<&PaceInsight>) -> PacePhrase {
    match pace {
        None => PacePhrase::Empty,
        Some(data) => match data.delta {
            d if d > 0 => PacePhrase::Up,
            d if d < 0 => PacePhrase::Down,
            _ => PacePhrase::Flat,
        },
    }
}

/// Фраза темпа выбранного периода данных.
pub(crate) fn pace_verdict_text(i18n: I18nContext<Locale>, pace: Option<&PaceInsight>) -> String {
    let locale = i18n.get_locale();
    let Some(data) = pace else {
        return td_string!(locale, insights.pace_empty).to_string();
    };
    let count = data.per_day;
    match pace_phrase_kind(Some(data)) {
        PacePhrase::Empty => td_string!(locale, insights.pace_empty).to_string(),
        PacePhrase::Up => td_string!(locale, insights.pace_up, count = count, delta = data.delta),
        PacePhrase::Down => {
            let magnitude = -data.delta;
            td_string!(locale, insights.pace_down, count = count, delta = magnitude)
        },
        PacePhrase::Flat => td_string!(locale, insights.pace_flat, count = count),
    }
}

/// Метка строки темпа.
pub(crate) fn pace_label(i18n: I18nContext<Locale>) -> String {
    let locale = i18n.get_locale();
    td_string!(locale, insights.pace_label).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn pace(delta: i32) -> PaceInsight {
        PaceInsight {
            per_day: 10,
            delta,
            recent_total: 140,
            previous_total: 70,
        }
    }

    #[rstest]
    #[case::no_data(None, PacePhrase::Empty)]
    #[case::growing(Some(pace(5)), PacePhrase::Up)]
    #[case::dropping(Some(pace(-6)), PacePhrase::Down)]
    #[case::flat(Some(pace(0)), PacePhrase::Flat)]
    fn pace_phrase_kind_maps_every_branch(
        #[case] input: Option<PaceInsight>,
        #[case] expected: PacePhrase,
    ) {
        assert_eq!(pace_phrase_kind(input.as_ref()), expected);
    }
}
