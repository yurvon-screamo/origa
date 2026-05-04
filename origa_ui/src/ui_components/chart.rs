use leptos::prelude::*;

const PADDING: u32 = 64;
const POINT_RADIUS: u32 = 4;

#[component]
pub fn LineChart(
    #[prop(into)] data: Signal<Vec<(String, f64)>>,
    #[prop(default = 400)] width: u32,
    #[prop(default = 200)] height: u32,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let chart_width = width - PADDING * 2;
    let chart_height = height - PADDING * 2;

    let is_flat_line = move || {
        let items = data.get();
        if items.is_empty() {
            return false;
        }
        let values: Vec<f64> = items.iter().map(|(_, v)| *v).collect();
        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        (max_val - min_val).abs() < 0.001
    };

    let normalized_points = move || {
        let items = data.get();
        if items.is_empty() {
            return Vec::new();
        }

        if items.len() == 1 {
            return vec![(
                width as f64 / 2.0,
                PADDING as f64 + chart_height as f64 / 2.0,
            )];
        }

        let values: Vec<f64> = items.iter().map(|(_, v)| *v).collect();
        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max_val - min_val).max(1.0);

        if items.len() == 2 {
            let usable_width = chart_width as f64 * 0.5;
            let offset_x = PADDING as f64 + chart_width as f64 * 0.25;
            return items
                .iter()
                .enumerate()
                .map(|(i, (_, value))| {
                    let x = offset_x + i as f64 * usable_width;
                    let y = PADDING as f64 + chart_height as f64
                        - ((value - min_val) / range) * chart_height as f64;
                    (x, y)
                })
                .collect::<Vec<_>>();
        }

        let step_x = chart_width as f64 / (items.len() - 1) as f64;

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

        if items.len() == 1 {
            return vec![(width as f64 / 2.0, items[0].0.clone())];
        }

        if items.len() == 2 {
            let usable_width = chart_width as f64 * 0.5;
            let offset_x = PADDING as f64 + chart_width as f64 * 0.25;
            return items
                .iter()
                .enumerate()
                .map(|(i, (label, _))| {
                    let x = offset_x + i as f64 * usable_width;
                    (x, label.clone())
                })
                .collect::<Vec<_>>();
        }

        let step_x = chart_width as f64 / (items.len() - 1) as f64;

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

        if (max_val - min_val).abs() < 0.001 {
            let y = PADDING as f64 + chart_height as f64 / 2.0;
            return vec![(y, min_val)];
        }

        let tick_count = if chart_height < 100 {
            2
        } else if chart_height < 160 {
            3
        } else {
            4
        };

        let mut ticks: Vec<(f64, f64)> = (0..=tick_count)
            .map(|i| {
                let ratio = i as f64 / tick_count as f64;
                let y = PADDING as f64 + chart_height as f64 * (1.0 - ratio);
                let value = min_val + (max_val - min_val) * ratio;
                (y, value)
            })
            .collect();

        let min_gap = 16.0f64;
        ticks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut deduped: Vec<(f64, f64)> = Vec::new();
        for (y, value) in ticks {
            if deduped
                .last()
                .map_or(true, |(last_y, _)| (y - *last_y).abs() >= min_gap)
            {
                deduped.push((y, value));
            }
        }

        deduped
    };

    let class_str = move || format!("{} chart-container w-full h-full", class.get());

    view! {
        <svg
            viewBox=format!("0 0 {} {}", width, height)
            class=class_str
            data-testid=test_id_val
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
                            x=PADDING - 28
                            y=y
                            text_anchor="end"
                            dominant_baseline="central"
                            fill="var(--fg-muted)"
                            font_size="9"
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
                            y=height - PADDING + 20
                            text_anchor="middle"
                            fill="var(--fg-muted)"
                            font_size="9"
                        >
                            {label}
                        </text>
                    }
                }
            />
            <polyline
                points=polyline_points
                fill="none"
                stroke="var(--accent-olive)"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            />
            <Show when=move || is_flat_line()>
                <line
                    x1=PADDING
                    y1=move || {
                        let items = data.get();
                        if items.is_empty() {
                            return (PADDING + chart_height / 2) as f64;
                        }
                        PADDING as f64 + chart_height as f64 / 2.0
                    }
                    x2=width - PADDING
                    y2=move || {
                        let items = data.get();
                        if items.is_empty() {
                            return (PADDING + chart_height / 2) as f64;
                        }
                        PADDING as f64 + chart_height as f64 / 2.0
                    }
                    stroke="var(--border-light)"
                    stroke-dasharray="4,4"
                />
            </Show>
            <Show when=move || !is_flat_line() || data.get().len() == 1>
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
            </Show>
        </svg>
    }
}
