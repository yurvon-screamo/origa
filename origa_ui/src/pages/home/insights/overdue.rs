//! Строка «Просрочка»: разница точности просроченных ревью и вовремя.

use crate::i18n::{I18nContext, Locale, td_string};
use origa::domain::insights::OverdueInsight;

/// Ветка фразы цены просрочки; чистая функция для прямого тестирования.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OverduePhrase {
    Empty,
    Costs,
    Free,
}

pub(crate) fn overdue_phrase_kind(insight: Option<&OverdueInsight>) -> OverduePhrase {
    match insight {
        None => OverduePhrase::Empty,
        Some(data) if data.accuracy_drop_pp > 0 => OverduePhrase::Costs,
        Some(_) => OverduePhrase::Free,
    }
}

/// Фраза цены просрочки.
pub(crate) fn overdue_verdict_text(
    i18n: I18nContext<Locale>,
    insight: Option<&OverdueInsight>,
) -> String {
    let locale = i18n.get_locale();
    let Some(data) = insight else {
        return td_string!(locale, insights.overdue_empty).to_string();
    };
    match overdue_phrase_kind(Some(data)) {
        OverduePhrase::Costs => {
            td_string!(locale, insights.overdue_costs, drop = data.accuracy_drop_pp)
        },
        _ => td_string!(locale, insights.overdue_free).to_string(),
    }
}

/// Метка строки просрочки.
pub(crate) fn overdue_label(i18n: I18nContext<Locale>) -> String {
    let locale = i18n.get_locale();
    td_string!(locale, insights.overdue_label).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn overdue(drop_pp: u8) -> OverdueInsight {
        OverdueInsight {
            accuracy_drop_pp: drop_pp,
            ontime_accuracy_pct: 86,
            late_accuracy_pct: 72,
        }
    }

    #[rstest]
    #[case::no_data(None, OverduePhrase::Empty)]
    #[case::costs(Some(overdue(30)), OverduePhrase::Costs)]
    #[case::free(Some(overdue(0)), OverduePhrase::Free)]
    fn overdue_phrase_kind_maps_every_branch(
        #[case] input: Option<OverdueInsight>,
        #[case] expected: OverduePhrase,
    ) {
        assert_eq!(overdue_phrase_kind(input.as_ref()), expected);
    }
}
