use crate::i18n::{I18nContext, Locale, td_string};

use super::panel_frame::verdict_class_by_kind;
use origa::domain::insights::{WeekVerdict, WeekVerdictKind};

/// Ветка фразы вердикта; чистая функция для прямого тестирования выбора.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WeekPhrase {
    Empty,
    Holding,
    Dipping,
}

pub(crate) fn week_phrase_kind(verdict: Option<&WeekVerdict>) -> WeekPhrase {
    match verdict {
        None => WeekPhrase::Empty,
        Some(data) if data.kind == WeekVerdictKind::Dipping => WeekPhrase::Dipping,
        Some(_) => WeekPhrase::Holding,
    }
}

/// Фраза вердикта выбранного периода: только факт. Числа обоих окон —
/// в `week_subline_text`, чтобы строка читалась одним взглядом.
pub(crate) fn week_verdict_text(
    i18n: I18nContext<Locale>,
    verdict: Option<&WeekVerdict>,
) -> String {
    let locale = i18n.get_locale();
    match week_phrase_kind(verdict) {
        WeekPhrase::Empty => td_string!(locale, insights.week_empty).to_string(),
        WeekPhrase::Dipping => td_string!(locale, insights.week_dipping).to_string(),
        WeekPhrase::Holding => td_string!(locale, insights.week_holding).to_string(),
    }
}

/// Sub-line вердикта: точность обоих окон сразу (неделя и месяц).
pub(crate) fn week_subline_text(
    i18n: I18nContext<Locale>,
    week: Option<&WeekVerdict>,
    month: Option<&WeekVerdict>,
) -> String {
    let locale = i18n.get_locale();
    let week_pct = week.map(|data| data.week_accuracy_pct);
    let month_pct = month.and_then(|data| data.month_accuracy_pct);
    match (week_pct, month_pct) {
        (Some(week), Some(month)) => {
            td_string!(locale, insights.week_subline, week = week, month = month)
        },
        (Some(week), None) => {
            td_string!(locale, insights.week_subline_no_month, week = week)
        },
        _ => String::new(),
    }
}

/// Класс фразы вердикта: serif как у остальных строк, окрашен по статусу.
pub(crate) fn week_verdict_class(verdict: Option<&WeekVerdict>) -> String {
    verdict_class_by_kind(verdict.map(|data| data.kind)).to_string()
}

/// Метка строки вердикта.
pub(crate) fn verdict_label(i18n: I18nContext<Locale>) -> String {
    let locale = i18n.get_locale();
    td_string!(locale, insights.verdict_label).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn verdict(kind: WeekVerdictKind, month: Option<u8>) -> WeekVerdict {
        WeekVerdict {
            week_accuracy_pct: 50,
            month_accuracy_pct: month,
            kind,
        }
    }

    #[rstest]
    #[case::no_data(None, WeekPhrase::Empty)]
    #[case::holding(Some(verdict(WeekVerdictKind::Holding, Some(70))), WeekPhrase::Holding)]
    #[case::dipping(Some(verdict(WeekVerdictKind::Dipping, Some(70))), WeekPhrase::Dipping)]
    fn week_phrase_kind_maps_every_branch(
        #[case] input: Option<WeekVerdict>,
        #[case] expected: WeekPhrase,
    ) {
        assert_eq!(week_phrase_kind(input.as_ref()), expected);
    }
}
