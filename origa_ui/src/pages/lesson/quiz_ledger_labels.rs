use crate::i18n::*;

use super::quiz_ledger::{LedgerRowKind, LedgerVerdict};

/// Localized verdict line label («Верно», «Есть ошибки», …) — the
/// uppercase styling is applied by the strip's CSS.
pub(crate) fn verdict_label(
    i18n: I18nContext<crate::i18n::Locale>,
    verdict: LedgerVerdict,
) -> String {
    match verdict {
        LedgerVerdict::Correct => t_string!(i18n, lesson.verdict_correct).to_string(),
        LedgerVerdict::LenientPass => t_string!(i18n, lesson.verdict_lenient).to_string(),
        LedgerVerdict::Errors => t_string!(i18n, lesson.verdict_errors).to_string(),
        LedgerVerdict::Incorrect => t_string!(i18n, lesson.verdict_incorrect).to_string(),
        LedgerVerdict::NotAnswered => t_string!(i18n, lesson.verdict_not_answered).to_string(),
    }
}

/// Localized status label of a ledger row («Верно», «Пропущено», …).
pub(crate) fn row_label(i18n: I18nContext<crate::i18n::Locale>, kind: LedgerRowKind) -> String {
    match kind {
        LedgerRowKind::SelectedCorrect => {
            t_string!(i18n, lesson.ledger_row_correct).to_string()
        },
        LedgerRowKind::YourAnswerCorrect | LedgerRowKind::YourAnswerWrong => {
            t_string!(i18n, lesson.ledger_row_yours).to_string()
        },
        LedgerRowKind::Missed => t_string!(i18n, lesson.ledger_row_missed).to_string(),
        LedgerRowKind::ExtraSelected => t_string!(i18n, lesson.ledger_row_extra).to_string(),
        LedgerRowKind::CorrectAnswer => {
            t_string!(i18n, lesson.ledger_row_correct_answer).to_string()
        },
    }
}

/// Yes/no answers are rendered as ДА/НЕТ in the ledger rows.
pub(crate) fn yesno_label(i18n: I18nContext<crate::i18n::Locale>, value: bool) -> String {
    if value {
        t_string!(i18n, lesson.yes).to_string()
    } else {
        t_string!(i18n, lesson.no).to_string()
    }
}
