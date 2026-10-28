//! Карточка «Точность за месяц» на home (fl2O037UR7): SVG-линия точности
//! по локальным дням за 30 дней. Дни без оценок — разрывы линии.

use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::insights::{AccuracySeries, CHART_DAYS, DayAccuracy};

/// Геометрия SVG: viewBox 600×170, поле слева под подписи оси.
const VIEW_W: f64 = 600.0;
const VIEW_H: f64 = 170.0;
const PLOT_LEFT: f64 = 36.0;
const PLOT_RIGHT: f64 = 592.0;
const PLOT_TOP: f64 = 8.0;
const PLOT_BOTTOM: f64 = 148.0;
const LABEL_BASELINE: f64 = VIEW_H - 4.0;

/// X точки по индексу дня (0 — месяц назад … 29 — сегодня).
fn x_for(index: usize) -> f64 {
    PLOT_LEFT + index as f64 * (PLOT_RIGHT - PLOT_LEFT) / (CHART_DAYS - 1) as f64
}

/// Y точки по проценту (0 внизу, 100 наверху).
fn y_for(pct: u8) -> f64 {
    PLOT_BOTTOM - f64::from(pct) / 100.0 * (PLOT_BOTTOM - PLOT_TOP)
}

/// Непрерывные сегменты линии: разрыв на днях без оценок.
fn accuracy_segments(days: &[DayAccuracy]) -> Vec<Vec<(f64, f64)>> {
    let mut segments: Vec<Vec<(f64, f64)>> = Vec::new();
    for (index, day) in days.iter().enumerate() {
        if let Some(pct) = day.accuracy_pct {
            let point = (x_for(index), y_for(pct));
            match segments.last_mut() {
                Some(segment) if index > 0 && days[index - 1].accuracy_pct.is_some() => {
                    segment.push(point)
                },
                _ => segments.push(vec![point]),
            }
        }
    }
    segments
}

/// Секции «Точность за месяц»: карточка с SVG-графиком.
#[component]
pub fn AccuracyChartCard(
    series: Signal<AccuracySeries>,
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

    let has_data = Signal::derive(move || series.get().has_data());
    let segments = Signal::derive(move || accuracy_segments(&series.get().days));
    let dots = Signal::derive(move || -> Vec<(f64, f64)> {
        series
            .get()
            .days
            .iter()
            .enumerate()
            .filter_map(|(index, day)| day.accuracy_pct.map(|pct| (x_for(index), y_for(pct))))
            .collect()
    });

    view! {
        <section data-testid=move || test_id.get()>
            <p class="text-xs text-[var(--fg-muted)] uppercase tracking-widest mb-4">
                {move || {
                    let locale = i18n.get_locale();
                    td_string!(locale, chart.section_title)
                }}
            </p>

            <div class="card card-shadow p-6">
                <Show
                    when=move || has_data.get()
                    fallback=move || {
                        view! {
                            <p
                                class="font-mono text-xs text-[var(--fg-muted)]"
                                data-testid=move || format!("{}-empty", section_prefix())
                            >
                                {move || {
                                    let locale = i18n.get_locale();
                                    td_string!(locale, chart.empty)
                                }}
                            </p>
                        }
                    }
                >
                    <svg
                        viewBox=format!("0 0 {VIEW_W} {VIEW_H}")
                        class="w-full h-auto"
                        role="img"
                        aria-label=move || {
                            let locale = i18n.get_locale();
                            td_string!(locale, chart.section_title)
                        }
                        data-testid=move || format!("{}-svg", section_prefix())
                    >
                        // Сетка 0/50/100%.
                        <line x1=PLOT_LEFT y1=PLOT_BOTTOM x2=PLOT_RIGHT y2=PLOT_BOTTOM stroke="var(--border-light)" stroke-width="1" />
                        <line x1=PLOT_LEFT y1=y_for(50) x2=PLOT_RIGHT y2=y_for(50) stroke="var(--border-light)" stroke-width="1" />
                        <line x1=PLOT_LEFT y1=PLOT_TOP x2=PLOT_RIGHT y2=PLOT_TOP stroke="var(--border-light)" stroke-width="1" />
                        <text x="0" y=PLOT_BOTTOM font-size="11" fill="var(--fg-muted)" style="font-family: var(--font-mono)">"0%"</text>
                        <text x="0" y=y_for(50) font-size="11" fill="var(--fg-muted)" style="font-family: var(--font-mono)">"50%"</text>
                        <text x="0" y=PLOT_TOP font-size="11" fill="var(--fg-muted)" style="font-family: var(--font-mono)">"100%"</text>

                        // Линия: сегменты между днями с оценками.
                        {move || {
                            segments
                                .get()
                                .iter()
                                .map(|segment| {
                                    let points = segment
                                        .iter()
                                        .map(|(x, y)| format!("{x:.1},{y:.1}"))
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    view! {
                                        <polyline
                                            points=points
                                            fill="none"
                                            stroke="var(--fg-black)"
                                            stroke-width="2"
                                            stroke-linejoin="round"
                                            stroke-linecap="round"
                                        />
                                    }
                                })
                                .collect_view()
                        }}

                        // Точки дней с оценками.
                        {move || {
                            dots
                                .get()
                                .iter()
                                .map(|(x, y)| {
                                    view! { <circle cx=*x cy=*y r="3" fill="var(--fg-black)" /> }
                                })
                                .collect_view()
                        }}

                        // Подписи оси X: границы окна.
                        <text x=PLOT_LEFT y=LABEL_BASELINE font-size="11" fill="var(--fg-muted)" style="font-family: var(--font-mono)">
                            {move || {
                                let locale = i18n.get_locale();
                                td_string!(locale, chart.month_ago)
                            }}
                        </text>
                        <text x=PLOT_RIGHT y=LABEL_BASELINE font-size="11" fill="var(--fg-muted)" text-anchor="end" style="font-family: var(--font-mono)">
                            {move || {
                                let locale = i18n.get_locale();
                                td_string!(locale, chart.today)
                            }}
                        </text>
                    </svg>
                </Show>
            </div>
        </section>
    }
}
