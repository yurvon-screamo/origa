use super::panel_frame::InsightPanel;
use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::insights::TimeOfDayInsight;

/// Панель «В какое время тяжелее?»: худшее 4-часовое окно.
#[component]
pub(crate) fn TimeOfDayPanel(
    insight: Signal<Option<TimeOfDayInsight>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let question = Signal::derive(move || {
        let locale = i18n.get_locale();
        td_string!(locale, insights.time_question)
    });

    let phrase = Signal::derive(move || {
        let locale = i18n.get_locale();
        let Some(data) = insight.get() else {
            return td_string!(locale, insights.collecting_data).to_string();
        };

        let period = match data.worst_bucket {
            0 => td_string!(locale, insights.time_period_0),
            1 => td_string!(locale, insights.time_period_1),
            2 => td_string!(locale, insights.time_period_2),
            3 => td_string!(locale, insights.time_period_3),
            4 => td_string!(locale, insights.time_period_4),
            _ => td_string!(locale, insights.time_period_5),
        };
        td_string!(locale, insights.time_worst, period = period)
    });

    let proofs = Signal::derive(move || {
        let locale = i18n.get_locale();
        let Some(data) = insight.get() else {
            return Vec::new();
        };
        vec![td_string!(
            locale,
            insights.time_worst_proof,
            accuracy = data.accuracy_pct,
            attempts = data.attempts
        )]
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
