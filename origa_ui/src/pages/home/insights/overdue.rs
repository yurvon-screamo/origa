use super::panel_frame::InsightPanel;
use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
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
        let Some(data) = insight.get() else {
            return td_string!(locale, insights.overdue_empty).to_string();
        };
        match overdue_phrase_kind(Some(&data)) {
            OverduePhrase::Costs => {
                td_string!(locale, insights.overdue_costs, drop = data.accuracy_drop_pp)
            },
            _ => td_string!(locale, insights.overdue_free).to_string(),
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
            has_data=Signal::derive(move || insight.get().is_some())
            proofs=proofs
            test_id=test_id
        />
    }
}
