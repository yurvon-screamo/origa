use super::panel_frame::InsightPanel;
use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::insights::HardestWordsInsight;

/// Панель «Какие слова самые тяжёлые?»: доля проблемных слов и топ провалов.
#[component]
pub(crate) fn HardestWordsPanel(
    insight: Signal<Option<HardestWordsInsight>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let question = Signal::derive(move || {
        let locale = i18n.get_locale();
        td_string!(locale, insights.words_question)
    });

    let phrase = Signal::derive(move || {
        let locale = i18n.get_locale();
        match insight.get() {
            None => td_string!(locale, insights.words_empty).to_string(),
            Some(data) => td_string!(
                locale,
                insights.words_troubled,
                troubled = data.troubled_count,
                total = data.studied_total,
                one_in = data.one_in
            ),
        }
    });

    let proofs = Signal::derive(move || {
        let locale = i18n.get_locale();
        let Some(data) = insight.get() else {
            return Vec::<String>::new();
        };
        let lapses_label = td_string!(locale, insights.words_proof_lapses);
        let lines: Vec<String> = data
            .top
            .iter()
            .map(|word| format!("{} — {} {}", word.japanese, word.lapses, lapses_label))
            .collect();
        if lines.is_empty() {
            return vec![td_string!(locale, insights.words_proof_empty).to_string()];
        }
        lines
    });

    view! {
        <InsightPanel
            question=question
            verdict=phrase
            has_data=Signal::derive(move || insight.get().is_some())
            proofs=proofs
            test_id=test_id
        />
    }
}
