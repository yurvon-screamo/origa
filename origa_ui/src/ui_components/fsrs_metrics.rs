use leptos::prelude::*;

use crate::i18n::use_i18n;
use crate::ui_components::Tooltip;

struct MetricData {
    fill_pct: f64,
    color_class: &'static str,
    aria_valuenow: Option<String>,
}

fn compute_metric_data(
    value: Option<f64>,
    fill_fn: fn(f64) -> f64,
    color_fn: &dyn Fn(f64, f64) -> &'static str,
    other_value: Option<f64>,
) -> MetricData {
    let fill_pct = value.map(fill_fn).unwrap_or(0.0);

    let color_class = match (value, other_value) {
        (Some(d), Some(s)) => color_fn(d, s),
        _ => "",
    };

    let aria_valuenow = value.map(|v| format!("{v:.1}"));

    MetricData {
        fill_pct,
        color_class,
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
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

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
