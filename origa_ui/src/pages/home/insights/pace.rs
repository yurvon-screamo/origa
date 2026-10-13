use super::panel_frame::InsightPanel;
use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::insights::PaceInsight;

/// Ветка фразы темпа; чистая функция для прямого тестирования выбора.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PacePhrase {
    Empty,
    Up,
    Down,
    Flat,
}

pub(crate) fn pace_phrase_kind(pace: Option<&PaceInsight>) -> PacePhrase {
    match pace {
        None => PacePhrase::Empty,
        Some(data) => match data.delta {
            d if d > 0 => PacePhrase::Up,
            d if d < 0 => PacePhrase::Down,
            _ => PacePhrase::Flat,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn pace(delta: i32) -> PaceInsight {
        PaceInsight {
            per_day: 10,
            delta,
            recent_total: 140,
            previous_total: 70,
        }
    }

    #[rstest]
    #[case::no_data(None, PacePhrase::Empty)]
    #[case::growing(Some(pace(5)), PacePhrase::Up)]
    #[case::dropping(Some(pace(-6)), PacePhrase::Down)]
    #[case::flat(Some(pace(0)), PacePhrase::Flat)]
    fn pace_phrase_kind_maps_every_branch(
        #[case] input: Option<PaceInsight>,
        #[case] expected: PacePhrase,
    ) {
        assert_eq!(pace_phrase_kind(input.as_ref()), expected);
    }
}

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
        match pace_phrase_kind(Some(&data)) {
            PacePhrase::Empty => td_string!(locale, insights.collecting_data).to_string(),
            PacePhrase::Up => {
                td_string!(locale, insights.pace_up, count = count, delta = data.delta)
            },
            PacePhrase::Down => {
                let magnitude = -data.delta;
                td_string!(locale, insights.pace_down, count = count, delta = magnitude)
            },
            PacePhrase::Flat => td_string!(locale, insights.pace_flat, count = count),
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
