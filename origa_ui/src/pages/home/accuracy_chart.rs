//! Карточка «Точность за месяц» на home (fl2O037UR7): линия точности
//! поверх общего `MultiLineChart` (наработки #528-era activity chart).
//! Точки — только дни с оценками, подписи «д мес» в локали юзера.

use crate::i18n::{td_string, use_i18n};
use crate::ui_components::{Card, ChartLine, MultiLineChart};
use chrono::Datelike;
use leptos::prelude::*;
use origa::domain::NativeLanguage;
use origa::domain::insights::AccuracySeries;

const RU_MONTHS_SHORT: [&str; 12] = [
    "янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
];
const EN_MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Подпись дня на оси X: «14 окт» / «14 Oct» — как в старом activity chart.
fn day_label(days_ago: u32, lang: &NativeLanguage) -> String {
    let date = chrono::Local::now().date_naive() - chrono::Duration::days(i64::from(days_ago));
    let month = match lang {
        NativeLanguage::Russian => RU_MONTHS_SHORT[date.month0() as usize],
        _ => EN_MONTHS_SHORT[date.month0() as usize],
    };
    format!("{} {}", date.day(), month)
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
        let data = series
            .get()
            .days
            .iter()
            .filter_map(|day| {
                day.accuracy_pct
                    .map(|pct| (day_label(day.days_ago, &lang), f64::from(pct)))
            })
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
