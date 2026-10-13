use super::panel_frame::InsightPanel;
use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::insights::{WeekVerdict, WeekVerdictKind};

/// Панель «Как идёт неделя?»: точность недели против предыдущих 30 дней.
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
            None => td_string!(locale, insights.collecting_data).to_string(),
            Some(data) => match data.kind {
                WeekVerdictKind::Dipping => {
                    let month = data.month_accuracy_pct.unwrap_or(0);
                    td_string!(
                        locale,
                        insights.week_dipping,
                        week = data.week_accuracy_pct,
                        month = month
                    )
                },
                WeekVerdictKind::Holding => match data.month_accuracy_pct {
                    Some(_) => {
                        td_string!(locale, insights.week_holding, week = data.week_accuracy_pct)
                    },
                    None => td_string!(
                        locale,
                        insights.week_holding_no_month,
                        week = data.week_accuracy_pct
                    ),
                },
            },
        }
    });

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
            proofs=proofs
            test_id=test_id
        />
    }
}
