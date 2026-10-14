//! Строка «Тяжёлые слова»: доля проблемных слов и топ провалов.

use crate::i18n::{I18nContext, Locale, td_string};
use origa::domain::insights::HardestWordsInsight;

/// Фраза тяжёлых слов.
pub(crate) fn words_verdict_text(
    i18n: I18nContext<Locale>,
    insight: Option<&HardestWordsInsight>,
) -> String {
    let locale = i18n.get_locale();
    match insight {
        None => td_string!(locale, insights.words_empty).to_string(),
        Some(data) => td_string!(
            locale,
            insights.words_troubled,
            troubled = data.troubled_count,
            total = data.studied_total,
            one_in = data.one_in
        ),
    }
}

/// Метка строки тяжёлых слов.
pub(crate) fn words_label(i18n: I18nContext<Locale>) -> String {
    let locale = i18n.get_locale();
    td_string!(locale, insights.words_label).to_string()
}
