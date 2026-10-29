//! Карточка «Точность за месяц» на home (fl2O037UR7): линия точности
//! поверх общего `MultiLineChart` (наработки #528-era activity chart).
//! Точки — только дни с оценками, подписи «д мес» в локали юзера.

use crate::i18n::{td_string, use_i18n};
use crate::ui_components::{Card, ChartLine, MultiLineChart};
use chrono::Datelike;
use leptos::prelude::*;
use origa::domain::NativeLanguage;
use origa::domain::insights::AccuracySeries;

use super::dashboard_stats::{EN_MONTHS_SHORT, RU_MONTHS_SHORT};

/// Подпись дня на оси X: «14 окт» / «14 Oct» — как в старом activity chart.
fn day_label(days_ago: u32, lang: &NativeLanguage, today: chrono::NaiveDate) -> String {
    let date = today - chrono::Duration::days(i64::from(days_ago));
    let month = match lang {
        NativeLanguage::Russian => RU_MONTHS_SHORT[date.month0() as usize],
        _ => EN_MONTHS_SHORT[date.month0() as usize],
    };
    format!("{} {}", date.day(), month)
}

/// Точки линии точности в порядке «от старых к новым»: только дни с
/// оценками (дни без повторений линию не рисуют).
fn chart_points(series: &AccuracySeries) -> Vec<(u32, u8)> {
    series
        .days
        .iter()
        .filter_map(|day| day.accuracy_pct.map(|pct| (day.days_ago, pct)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::insights::{CHART_DAYS, DayAccuracy};

    fn series_with(reviews: &[(usize, u8)]) -> AccuracySeries {
        let days = (0..CHART_DAYS)
            .map(|index| {
                let days_ago = (CHART_DAYS - 1 - index) as u32;
                let review = reviews.iter().find(|(idx, _)| *idx == index);
                DayAccuracy {
                    days_ago,
                    accuracy_pct: review.map(|(_, pct)| *pct),
                }
            })
            .collect();
        AccuracySeries { days }
    }

    #[test]
    fn chart_points_keeps_only_reviewed_days_oldest_first() {
        // Индексы 0..29 от старых к новым: index 2 → days_ago 27.
        let series = series_with(&[(2, 90), (3, 80), (10, 60)]);
        let points = chart_points(&series);
        assert_eq!(points, vec![(27, 90), (26, 80), (19, 60)]);
    }

    #[test]
    fn chart_points_empty_series_is_empty() {
        assert!(chart_points(&series_with(&[])).is_empty());
    }

    #[test]
    fn day_label_formats_ru_and_en() {
        use chrono::TimeZone;
        let today = chrono::Utc
            .with_ymd_and_hms(2026, 10, 14, 0, 0, 0)
            .unwrap()
            .date_naive();
        assert_eq!(day_label(0, &NativeLanguage::Russian, today), "14 окт");
        assert_eq!(day_label(1, &NativeLanguage::English, today), "13 Oct");
    }
}

/// Карточка «Точность за месяц»: MultiLineChart с одной линией точности.
#[component]
pub fn AccuracyChartCard(
    series: Signal<AccuracySeries>,
    lang: Signal<NativeLanguage>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let section_prefix = move || {
        let val = test_id.get();
        if val.is_empty() {
            "home-accuracy-chart".to_string()
        } else {
            val
        }
    };

    let lines = Signal::derive(move || {
        let lang = lang.get();
        let today = chrono::Local::now().date_naive();
        let data = chart_points(&series.get())
            .into_iter()
            .map(|(days_ago, pct)| (day_label(days_ago, &lang, today), f64::from(pct)))
            .collect();
        vec![ChartLine {
            data,
            color: "var(--fg-black)".to_string(),
            label: td_string!(i18n.get_locale(), chart.line_label).to_string(),
        }]
    });

    let empty_text = Signal::derive(move || td_string!(i18n.get_locale(), chart.empty).to_string());

    view! {
        <section data-testid=move || test_id.get()>
            <p class="text-xs text-[var(--fg-muted)] uppercase tracking-widest mb-4">
                {move || {
                    let locale = i18n.get_locale();
                    td_string!(locale, chart.section_title)
                }}
            </p>

            <Card shadow=true class=Signal::derive(|| "p-6".to_string())>
                <MultiLineChart
                    lines=lines
                    width=600
                    height=280
                    empty_text=empty_text
                    test_id=Signal::derive(move || format!("{}-chart", section_prefix()))
                />
            </Card>
        </section>
    }
}
