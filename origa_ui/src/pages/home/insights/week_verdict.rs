use super::panel_frame::InsightPanel;
use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::insights::{WeekVerdict, WeekVerdictKind};

/// Ветка фразы вердикта недели; выделена в чистую функцию для прямого
/// тестирования выбора (маппинг ветки на i18n-ключ тривиален).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WeekPhrase {
    Empty,
    Holding,
    HoldingNoMonth,
    Dipping,
}

pub(crate) fn week_phrase_kind(verdict: Option<&WeekVerdict>) -> WeekPhrase {
    match verdict {
        None => WeekPhrase::Empty,
        Some(data) => match data.kind {
            WeekVerdictKind::Dipping => WeekPhrase::Dipping,
            WeekVerdictKind::Holding => match data.month_accuracy_pct {
                Some(_) => WeekPhrase::Holding,
                None => WeekPhrase::HoldingNoMonth,
            },
        },
    }
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
    #[case::holding_with_month(
        Some(verdict(WeekVerdictKind::Holding, Some(70))),
        WeekPhrase::Holding
    )]
    #[case::holding_without_month(
        Some(verdict(WeekVerdictKind::Holding, None)),
        WeekPhrase::HoldingNoMonth
    )]
    #[case::dipping(Some(verdict(WeekVerdictKind::Dipping, Some(70))), WeekPhrase::Dipping)]
    fn week_phrase_kind_maps_every_branch(
        #[case] input: Option<WeekVerdict>,
        #[case] expected: WeekPhrase,
    ) {
        assert_eq!(week_phrase_kind(input.as_ref()), expected);
    }
}

/// Цвет главной фразы по вердикту: держишь темп — оливковый, просел —
/// терракотовый (ошибка). Пустые данные — приглушённый серый.
fn week_verdict_class(kind: Option<WeekVerdictKind>) -> &'static str {
    match kind {
        None => "heading-h2 text-[var(--fg-muted)]",
        Some(WeekVerdictKind::Dipping) => "heading-h2 text-[var(--error)]",
        Some(WeekVerdictKind::Holding) => "heading-h2 text-[var(--success)]",
    }
}

/// Featured-панель «Как идёт неделя?»: главный вердикт секции, вся ширина.
#[component]
pub(crate) fn WeekVerdictPanel(
    verdict: Signal<Option<WeekVerdict>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let question = Signal::derive(move || {
        let locale = i18n.get_locale();
        td_string!(locale, insights.week_question)
    });

    let phrase = Signal::derive(move || {
        let locale = i18n.get_locale();
        match verdict.get() {
            None => td_string!(locale, insights.week_empty).to_string(),
            Some(data) => match week_phrase_kind(Some(&data)) {
                WeekPhrase::Empty => td_string!(locale, insights.week_empty).to_string(),
                WeekPhrase::Dipping => {
                    let month = data.month_accuracy_pct.unwrap_or(0);
                    td_string!(
                        locale,
                        insights.week_dipping,
                        week = data.week_accuracy_pct,
                        month = month
                    )
                },
                WeekPhrase::Holding => {
                    td_string!(locale, insights.week_holding, week = data.week_accuracy_pct)
                },
                WeekPhrase::HoldingNoMonth => td_string!(
                    locale,
                    insights.week_holding_no_month,
                    week = data.week_accuracy_pct
                ),
            },
        }
    });

    let verdict_class =
        Signal::derive(move || week_verdict_class(verdict.get().map(|data| data.kind)).to_string());

    let proofs = Signal::derive(move || {
        let locale = i18n.get_locale();
        let Some(data) = verdict.get() else {
            return Vec::new();
        };

        let week_line = format!(
            "{}: {}%",
            td_string!(locale, insights.week_proof_week),
            data.week_accuracy_pct
        );
        let month_value = data
            .month_accuracy_pct
            .map(|month| format!("{month}%"))
            .unwrap_or_else(|| td_string!(locale, insights.week_proof_month_absent).to_string());
        let month_line = format!(
            "{}: {}",
            td_string!(locale, insights.week_proof_month),
            month_value
        );
        vec![week_line, month_line]
    });

    view! {
        <InsightPanel
            question=question
            verdict=phrase
            verdict_class=verdict_class
            has_data=Signal::derive(move || verdict.get().is_some())
            proofs=proofs
            test_id=test_id
        />
    }
}
