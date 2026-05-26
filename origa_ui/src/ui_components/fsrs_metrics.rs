use leptos::prelude::*;

use crate::i18n::use_i18n;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum FsrsMetricsMode {
    #[default]
    Compact,
    Expanded,
}

struct MetricData {
    value_display: String,
    fill_pct: f64,
    color_class: &'static str,
    value_class: &'static str,
    aria_valuenow: Option<String>,
}

fn compute_metric_data(
    value: Option<f64>,
    fill_fn: fn(f64) -> f64,
    color_fn: &dyn Fn(f64, f64) -> &'static str,
    other_value: Option<f64>,
) -> MetricData {
    let value_display = value
        .map(|v| format!("{v:.1}"))
        .unwrap_or_else(|| "\u{2014}".to_string());

    let fill_pct = value.map(fill_fn).unwrap_or(0.0);

    let color_class = match (value, other_value) {
        (Some(d), Some(s)) => color_fn(d, s),
        _ => "",
    };

    let value_class = if value.is_none() {
        "fsrs-value fsrs-value--none"
    } else {
        "fsrs-value"
    };

    let aria_valuenow = value.map(|v| format!("{v:.1}"));

    MetricData {
        value_display,
        fill_pct,
        color_class,
        value_class,
        aria_valuenow,
    }
}

fn difficulty_fill(d: f64) -> f64 {
    (d / 10.0).min(1.0) * 100.0
}

fn stability_fill(s: f64) -> f64 {
    (s / 30.0).min(1.0) * 100.0
}

fn difficulty_color_class(d: f64, s: f64) -> &'static str {
    if d >= 7.0 && s < 7.0 {
        "fsrs-bar--terracotta"
    } else if d >= 4.0 {
        "fsrs-bar--gold"
    } else {
        "fsrs-bar--olive"
    }
}

fn stability_color_class(s: f64) -> &'static str {
    if s > 21.0 {
        "fsrs-bar--olive"
    } else if s >= 7.0 {
        "fsrs-bar--gold"
    } else {
        "fsrs-bar--terracotta"
    }
}

#[component]
pub fn FsrsMetrics(
    difficulty: Option<f64>,
    stability: Option<f64>,
    #[prop(optional)] next_review_date: Option<String>,
    #[prop(optional, default = FsrsMetricsMode::Compact)] mode: FsrsMetricsMode,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    match mode {
        FsrsMetricsMode::Compact => view! {
            <span class="fsrs-metrics" role="group" aria-label="FSRS metrics" data-testid=test_id_val>
                <CompactMetric
                    label="D"
                    value=difficulty
                    fill_fn=difficulty_fill
                    color_fn=move |d, s| difficulty_color_class(d, s)
                    other_value=stability
                    aria_label="Difficulty"
                    aria_max="10"
                />
                <CompactMetric
                    label="S"
                    value=stability
                    fill_fn=stability_fill
                    color_fn=move |_, s| stability_color_class(s)
                    other_value=difficulty
                    aria_label="Stability"
                    aria_max="30"
                />
            </span>
        }
        .into_any(),
        FsrsMetricsMode::Expanded => {
            let next_review_label = i18n
                .get_keys()
                .shared()
                .fsrs_next_review()
                .inner()
                .to_string();
            let difficulty_label = i18n
                .get_keys()
                .shared()
                .fsrs_difficulty()
                .inner()
                .to_string();
            let stability_label = i18n
                .get_keys()
                .shared()
                .fsrs_stability()
                .inner()
                .to_string();

            view! {
                <div
                    class="fsrs-metrics fsrs-metrics--expanded"
                    role="group"
                    aria-label="FSRS metrics"
                    data-testid=test_id_val
                >
                    {next_review_date
                        .map(|date| {
                            view! {
                                <div class="fsrs-metric--expanded">
                                    <span class="fsrs-label">{next_review_label.clone()}</span>
                                    <span class="fsrs-value fsrs-value--date">{date}</span>
                                </div>
                            }
                                .into_any()
                        })}
                    <ExpandedMetric
                        label=difficulty_label
                        value=difficulty
                        fill_fn=difficulty_fill
                        color_fn=move |d, s| difficulty_color_class(d, s)
                        other_value=stability
                        aria_label="Difficulty"
                        aria_max="10"
                    />
                    <ExpandedMetric
                        label=stability_label
                        value=stability
                        fill_fn=stability_fill
                        color_fn=move |_, s| stability_color_class(s)
                        other_value=difficulty
                        aria_label="Stability"
                        aria_max="30"
                    />
                </div>
            }
                .into_any()
        }
    }
}

#[component]
fn CompactMetric(
    #[prop(into)] label: &'static str,
    value: Option<f64>,
    fill_fn: fn(f64) -> f64,
    color_fn: impl Fn(f64, f64) -> &'static str + 'static,
    other_value: Option<f64>,
    #[prop(into)] aria_label: &'static str,
    #[prop(into)] aria_max: &'static str,
) -> impl IntoView {
    let data = compute_metric_data(value, fill_fn, &color_fn, other_value);

    view! {
        <span
            class="fsrs-metric"
            role="meter"
            aria-label=aria_label
            aria-valuenow=data.aria_valuenow
            aria-valuemin="0"
            aria-valuemax=aria_max
        >
            <span class="fsrs-label">{label}</span>
            <span
                class=format!("fsrs-bar {color_class}", color_class = data.color_class)
                style=format!("--fsrs-fill: {fill_pct:.0}%", fill_pct = data.fill_pct)
            ></span>
            <span class=data.value_class>{data.value_display}</span>
        </span>
    }
}

#[component]
fn ExpandedMetric(
    #[prop(into)] label: String,
    value: Option<f64>,
    fill_fn: fn(f64) -> f64,
    color_fn: impl Fn(f64, f64) -> &'static str + 'static,
    other_value: Option<f64>,
    #[prop(into)] aria_label: &'static str,
    #[prop(into)] aria_max: &'static str,
) -> impl IntoView {
    let data = compute_metric_data(value, fill_fn, &color_fn, other_value);

    view! {
        <div class="fsrs-metric--expanded">
            <span class="fsrs-label">{label}</span>
            <div
                class="fsrs-metric-row"
                role="meter"
                aria-label=aria_label
                aria-valuenow=data.aria_valuenow
                aria-valuemin="0"
                aria-valuemax=aria_max
            >
                <span
                    class=format!(
                        "fsrs-bar fsrs-bar--expanded {color_class}",
                        color_class = data.color_class,
                    )
                    style=format!("--fsrs-fill: {fill_pct:.0}%", fill_pct = data.fill_pct)
                ></span>
                <span class=data.value_class>{data.value_display}</span>
            </div>
        </div>
    }
}
