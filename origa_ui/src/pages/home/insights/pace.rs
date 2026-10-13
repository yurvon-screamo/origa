use super::panel_frame::InsightPanel;
use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::insights::PaceInsight;

/// Панель «Каков темп?»: среднее повторений за день против прошлых 14 дней.
#[component]
pub(crate) fn PacePanel(
    pace: Signal<Option<PaceInsight>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let question = Signal::derive(move || {
        let locale = i18n.get_locale();
        td_string!(locale, insights.pace_question)
    });

    let phrase = Signal::derive(move || {
        let locale = i18n.get_locale();
        let Some(data) = pace.get() else {
            return td_string!(locale, insights.collecting_data).to_string();
        };

        let count = data.per_day;
        match data.delta {
            d if d > 0 => td_string!(locale, insights.pace_up, count = count, delta = data.delta),
            d if d < 0 => {
                let magnitude = -data.delta;
                td_string!(locale, insights.pace_down, count = count, delta = magnitude)
            },
            _ => td_string!(locale, insights.pace_flat, count = count),
        }
    });

    let proofs = Signal::derive(move || {
        let locale = i18n.get_locale();
        let Some(data) = pace.get() else {
            return Vec::new();
        };
        vec![
            format!(
                "{}: {}",
                td_string!(locale, insights.pace_proof_recent),
                data.recent_total
            ),
            format!(
                "{}: {}",
                td_string!(locale, insights.pace_proof_previous),
                data.previous_total
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
