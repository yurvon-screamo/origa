use super::panel_frame::InsightPanel;
use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::insights::TimeOfDayInsight;

/// Ветка фразы времени суток; чистая функция для прямого тестирования
/// выбора (маппинг ветки на i18n-ключ тривиален).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimePhrase {
    Empty,
    Worst,
}

pub(crate) fn time_phrase_kind(insight: Option<&TimeOfDayInsight>) -> TimePhrase {
    match insight {
        None => TimePhrase::Empty,
        Some(_) => TimePhrase::Worst,
    }
}

/// Индекс ключа периода (0..6) для ветки Worst; выход за 5 клампится.
pub(crate) fn time_period_index(bucket: usize) -> usize {
    bucket.min(5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::no_data(None, TimePhrase::Empty)]
    #[case::worst(Some(TimeOfDayInsight {
        worst_bucket: 5,
        accuracy_pct: 52,
        attempts: 23,
    }), TimePhrase::Worst)]
    fn time_phrase_kind_maps_every_branch(
        #[case] input: Option<TimeOfDayInsight>,
        #[case] expected: TimePhrase,
    ) {
        assert_eq!(time_phrase_kind(input.as_ref()), expected);
    }

    #[rstest]
    #[case(0, 0)]
    #[case(3, 3)]
    #[case(5, 5)]
    #[case(9, 5)]
    fn time_period_index_clamps_to_last_window(#[case] bucket: usize, #[case] expected: usize) {
        assert_eq!(time_period_index(bucket), expected);
    }
}

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
            return td_string!(locale, insights.time_empty).to_string();
        };
        match time_phrase_kind(Some(&data)) {
            TimePhrase::Empty => td_string!(locale, insights.time_empty).to_string(),
            TimePhrase::Worst => {
                let period = match time_period_index(data.worst_bucket) {
                    0 => td_string!(locale, insights.time_period_0),
                    1 => td_string!(locale, insights.time_period_1),
                    2 => td_string!(locale, insights.time_period_2),
                    3 => td_string!(locale, insights.time_period_3),
                    4 => td_string!(locale, insights.time_period_4),
                    _ => td_string!(locale, insights.time_period_5),
                };
                td_string!(locale, insights.time_worst, period = period)
            },
        }
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
            has_data=Signal::derive(move || insight.get().is_some())
            proofs=proofs
            test_id=test_id
        />
    }
}
