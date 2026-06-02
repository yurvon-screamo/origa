use leptos::prelude::*;

use crate::i18n::use_i18n;
use crate::ui_components::Tooltip;

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

fn difficulty_color_class(d: f64) -> &'static str {
    if d >= 7.0 {
        "fsrs-bar--terracotta"
    } else if d >= 4.0 {
        "fsrs-bar--gold"
    } else {
        "fsrs-bar--olive"
    }
}

fn stability_color_class(s: f64) -> &'static str {
    if s >= 21.0 {
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
        FsrsMetricsMode::Compact => {
            let difficulty_short = Signal::derive(move || {
                i18n.get_keys()
                    .shared()
                    .fsrs_difficulty_short()
                    .inner()
                    .to_string()
            });
            let stability_short = Signal::derive(move || {
                i18n.get_keys()
                    .shared()
                    .fsrs_stability_short()
                    .inner()
                    .to_string()
            });
            let difficulty_full = i18n
                .get_keys()
                .shared()
                .fsrs_difficulty()
                .inner()
                .to_string();
            let stability_full = i18n
                .get_keys()
                .shared()
                .fsrs_stability()
                .inner()
                .to_string();

            view! {
                <span class="fsrs-metrics" role="group" aria-label="FSRS metrics" data-testid=test_id_val>
                    <CompactMetric
                        label=difficulty_short
                        tooltip_text=difficulty_full
                        value=difficulty
                        fill_fn=difficulty_fill
                        color_fn=move |d, _| difficulty_color_class(d)
                        other_value=stability
                        aria_label="Difficulty"
                        aria_max="10"
                    />
                    <CompactMetric
                        label=stability_short
                        tooltip_text=stability_full
                        value=stability
                        fill_fn=stability_fill
                        color_fn=move |_, s| stability_color_class(s)
                        other_value=difficulty
                        aria_label="Stability"
                        aria_max="30"
                    />
                </span>
            }
                .into_any()
        },
        FsrsMetricsMode::Expanded => {
            let next_review_label = Signal::derive(move || {
                i18n.get_keys()
                    .shared()
                    .fsrs_next_review()
                    .inner()
                    .to_string()
            });
            let difficulty_label = Signal::derive(move || {
                i18n.get_keys()
                    .shared()
                    .fsrs_difficulty()
                    .inner()
                    .to_string()
            });
            let stability_label = Signal::derive(move || {
                i18n.get_keys()
                    .shared()
                    .fsrs_stability()
                    .inner()
                    .to_string()
            });

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
                                    <span class="fsrs-label">{move || next_review_label.get()}</span>
                                    <span class="fsrs-value fsrs-value--date">{date}</span>
                                </div>
                            }
                                .into_any()
                        })}
                    <ExpandedMetric
                        label=difficulty_label
                        value=difficulty
                        fill_fn=difficulty_fill
                        color_fn=move |d, _| difficulty_color_class(d)
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
        },
    }
}

#[component]
fn CompactMetric(
    #[prop(into)] label: Signal<String>,
    #[prop(into)] tooltip_text: String,
    value: Option<f64>,
    fill_fn: fn(f64) -> f64,
    color_fn: impl Fn(f64, f64) -> &'static str + 'static,
    other_value: Option<f64>,
    #[prop(into)] aria_label: &'static str,
    #[prop(into)] aria_max: &'static str,
) -> impl IntoView {
    let data = compute_metric_data(value, fill_fn, &color_fn, other_value);

    view! {
        <Tooltip text=Signal::derive(move || tooltip_text.clone())>
            <span
                class="fsrs-metric"
                role="meter"
                aria-label=aria_label
                aria-valuenow=data.aria_valuenow
                aria-valuemin="0"
                aria-valuemax=aria_max
            >
                <span class="fsrs-label">{move || label.get()}</span>
                <span
                    class=format!("fsrs-bar {color_class}", color_class = data.color_class)
                    style=format!("--fsrs-fill: {fill_pct:.0}%", fill_pct = data.fill_pct)
                ></span>
            </span>
        </Tooltip>
    }
}

#[component]
fn ExpandedMetric(
    #[prop(into)] label: Signal<String>,
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
            <span class="fsrs-label">{move || label.get()}</span>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_color_class() {
        assert_eq!(difficulty_color_class(8.0), "fsrs-bar--terracotta");
        assert_eq!(difficulty_color_class(7.0), "fsrs-bar--terracotta");
        assert_eq!(difficulty_color_class(5.0), "fsrs-bar--gold");
        assert_eq!(difficulty_color_class(4.0), "fsrs-bar--gold");
        assert_eq!(difficulty_color_class(2.0), "fsrs-bar--olive");
        assert_eq!(difficulty_color_class(0.0), "fsrs-bar--olive");
    }

    #[test]
    fn test_stability_color_class() {
        assert_eq!(stability_color_class(25.0), "fsrs-bar--olive");
        assert_eq!(stability_color_class(21.0), "fsrs-bar--olive");
        assert_eq!(stability_color_class(10.0), "fsrs-bar--gold");
        assert_eq!(stability_color_class(7.0), "fsrs-bar--gold");
        assert_eq!(stability_color_class(3.0), "fsrs-bar--terracotta");
        assert_eq!(stability_color_class(0.0), "fsrs-bar--terracotta");
    }

    #[test]
    fn test_difficulty_fill() {
        assert_eq!(difficulty_fill(5.0), 50.0);
        assert_eq!(difficulty_fill(10.0), 100.0);
        assert_eq!(difficulty_fill(15.0), 100.0);
        assert_eq!(difficulty_fill(0.0), 0.0);
    }

    #[test]
    fn test_stability_fill() {
        assert_eq!(stability_fill(15.0), 50.0);
        assert_eq!(stability_fill(30.0), 100.0);
        assert_eq!(stability_fill(45.0), 100.0);
        assert_eq!(stability_fill(0.0), 0.0);
    }
}
