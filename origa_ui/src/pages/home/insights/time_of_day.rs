//! Строка «Худшее время»: худшее 4-часовое окно по точности.

use crate::i18n::{I18nContext, Locale, td_string};
use origa::domain::insights::TimeOfDayInsight;

/// Индекс ключа периода (0..6) для ветки Worst; выход за 5 клампится.
pub(crate) fn time_period_index(bucket: usize) -> usize {
    bucket.min(5)
}

/// Фраза худшего времени суток.
pub(crate) fn time_verdict_text(
    i18n: I18nContext<Locale>,
    insight: Option<&TimeOfDayInsight>,
) -> String {
    let locale = i18n.get_locale();
    let Some(data) = insight else {
        return td_string!(locale, insights.time_empty).to_string();
    };
    let period = match time_period_index(data.worst_bucket) {
        0 => td_string!(locale, insights.time_period_0),
        1 => td_string!(locale, insights.time_period_1),
        2 => td_string!(locale, insights.time_period_2),
        3 => td_string!(locale, insights.time_period_3),
        4 => td_string!(locale, insights.time_period_4),
        _ => td_string!(locale, insights.time_period_5),
    };
    td_string!(locale, insights.time_worst, period = period)
}

/// Sub-line худшего окна: объём выборки и точность в нём.
pub(crate) fn time_subline_text(
    i18n: I18nContext<Locale>,
    insight: Option<&TimeOfDayInsight>,
) -> String {
    let locale = i18n.get_locale();
    let Some(data) = insight else {
        return String::new();
    };
    td_string!(
        locale,
        insights.time_sub,
        attempts = data.attempts,
        accuracy = data.accuracy_pct
    )
    .to_string()
}

/// Метка строки худшего времени.
pub(crate) fn time_label(i18n: I18nContext<Locale>) -> String {
    let locale = i18n.get_locale();
    td_string!(locale, insights.time_label).to_string()
}

#[cfg(test)]
mod tests {
    use super::time_period_index;
    use rstest::rstest;

    #[rstest]
    #[case(0, 0)]
    #[case(3, 3)]
    #[case(5, 5)]
    #[case(9, 5)]
    fn time_period_index_clamps_to_last_window(#[case] bucket: usize, #[case] expected: usize) {
        assert_eq!(time_period_index(bucket), expected);
    }
}
