use leptos::prelude::*;

const PADDING: u32 = 40;
const POINT_RADIUS: u32 = 4;

#[component]
pub fn LineChart(
    #[prop(into)] data: Signal<Vec<(String, f64)>>,
    #[prop(default = 400)] width: u32,
    #[prop(default = 200)] height: u32,
    #[prop(optional, into)] class: Signal<String>,
) -> impl IntoView {
    let chart_width = width - PADDING * 2;
    let chart_height = height - PADDING * 2;

    let normalized_points = move || {
        let items = data.get();
        if items.is_empty() {
            return Vec::new();
        }

        let values: Vec<f64> = items.iter().map(|(_, v)| *v).collect();
        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max_val - min_val).max(1.0);

        let step_x = if items.len() > 1 {
            chart_width as f64 / (items.len() - 1) as f64
        } else {
            0.0
        };

        items
            .iter()
            .enumerate()
            .map(|(i, (_, value))| {
                let x = PADDING as f64 + i as f64 * step_x;
                let y = PADDING as f64 + chart_height as f64
                    - ((value - min_val) / range) * chart_height as f64;
                (x, y)
            })
            .collect::<Vec<_>>()
    };

    let polyline_points = move || {
        normalized_points()
            .iter()
            .map(|(x, y)| format!("{:.1},{:.1}", x, y))
            .collect::<Vec<_>>()
            .join(" ")
    };

    let x_labels = move || {
        let items = data.get();
        if items.is_empty() {
            return Vec::new();
        }

        let step_x = if items.len() > 1 {
            chart_width as f64 / (items.len() - 1) as f64
        } else {
            0.0
        };

        items
            .iter()
            .enumerate()
            .map(|(i, (label, _))| {
                let x = PADDING as f64 + i as f64 * step_x;
                (x, label.clone())
            })
            .collect::<Vec<_>>()
    };

    let y_ticks = move || {
        let items = data.get();
        if items.is_empty() {
            return Vec::new();
        }

        let values: Vec<f64> = items.iter().map(|(_, v)| *v).collect();
        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        (0..=4)
            .map(|i| {
                let ratio = i as f64 / 4.0;
                let y = PADDING as f64 + chart_height as f64 * (1.0 - ratio);
                let value = min_val + (max_val - min_val) * ratio;
                (y, value)
            })
            .collect::<Vec<_>>()
    };

    let class_str = move || class.get();

    view! {
        <svg
            width=width
            height=height
            class=class_str
            style="font-family: 'DM Mono', monospace; font-size: 10px;"
        >
            <line
                x1=PADDING
                y1=PADDING
                x2=PADDING
                y2=height - PADDING
                stroke="var(--fg-muted)"
                stroke-width="1"
            />
            <line
                x1=PADDING
                y1=height - PADDING
                x2=width - PADDING
                y2=height - PADDING
                stroke="var(--fg-muted)"
                stroke-width="1"
            />
            <For
                each=move || y_ticks()
                key=|(_, v)| v.to_bits()
                children=move |(y, value)| {
                    view! {
                        <text
                            x=PADDING - 8
                            y=y
                            text_anchor="end"
                            dominant_baseline="middle"
                            fill="var(--fg-muted)"
                        >
                            {format!("{:.0}", value)}
                        </text>
                        <line
                            x1=PADDING - 4
                            y1=y
                            x2=PADDING
                            y2=y
                            stroke="var(--fg-muted)"
                            stroke-width="0.5"
                        />
                    }
                }
            />
            <For
                each=move || x_labels()
                key=|(_, label)| label.clone()
                children=move |(x, label)| {
                    view! {
                        <text
                            x=x
                            y=height - PADDING + 16
                            text_anchor="middle"
                            fill="var(--fg-muted)"
                        >
                            {label}
                        </text>
                    }
                }
            />
            <polyline
                points=move || polyline_points()
                fill="none"
                stroke="var(--accent-olive)"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            />
            <For
                each=move || normalized_points()
                key=|(x, y)| ((*x * 1000.0) as i64, (*y * 1000.0) as i64)
                children=move |(x, y)| {
                    view! {
                        <circle
                            cx=x
                            cy=y
                            r=POINT_RADIUS
                            fill="var(--accent-olive)"
                            stroke="var(--bg-paper)"
                            stroke-width="2"
                        />
                    }
                }
            />
        </svg>
    }
}
