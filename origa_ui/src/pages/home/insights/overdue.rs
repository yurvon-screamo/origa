use super::panel_frame::InsightPanel;
use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::insights::OverdueInsight;

/// Панель «Сколько стоит просрочка?»: падение точности у просроченных ревью.
#[component]
pub(crate) fn OverduePanel(
    insight: Signal<Option<OverdueInsight>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let question = Signal::derive(move || {
        let locale = i18n.get_locale();
        td_string!(locale, insights.overdue_question)
    });

    let phrase = Signal::derive(move || {
        let locale = i18n.get_locale();
        match insight.get() {
            None => td_string!(locale, insights.collecting_data).to_string(),
            Some(data) if data.accuracy_drop_pp > 0 => {
                td_string!(locale, insights.overdue_costs, drop = data.accuracy_drop_pp)
            },
            Some(_) => td_string!(locale, insights.overdue_free).to_string(),
        }
    });

    let proofs = Signal::derive(move || {
        let locale = i18n.get_locale();
        let Some(data) = insight.get() else {
            return Vec::new();
        };
        vec![
            td_string!(
                locale,
                insights.overdue_proof_ontime,
                accuracy = data.ontime_accuracy_pct
            ),
            td_string!(
                locale,
                insights.overdue_proof_late,
                accuracy = data.late_accuracy_pct
            ),
        ]
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
