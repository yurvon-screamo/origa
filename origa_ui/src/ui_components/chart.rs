use leptos::prelude::*;

const PADDING: u32 = 32;
const POINT_RADIUS: u32 = 4;
const MAX_X_LABELS: usize = 6;

/// Pick the indices of X-axis labels to render so they never crowd.
/// When `count <= max_labels` every index is kept; otherwise every
/// `ceil(count / max_labels)`-th index is shown (0, step, 2*step, ...).
/// The last index is always appended when missing so the most recent
/// data point stays labeled.
pub(crate) fn select_label_indices(count: usize, max_labels: usize) -> Vec<usize> {
    if count == 0 || max_labels == 0 {
        return Vec::new();
    }
    if count <= max_labels {
        return (0..count).collect();
    }
    let step = ((count as f64) / (max_labels as f64)).ceil() as usize;
    let mut indices: Vec<usize> = (0..count).step_by(step.max(1)).collect();
    if indices.last() != Some(&(count - 1)) {
        indices.push(count - 1);
    }
    indices
}

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

        let step_x = chart_width as f64 / (items.len() - 1) as f64;

        select_label_indices(items.len(), MAX_X_LABELS)
            .into_iter()
            .map(|i| {
                let x = PADDING as f64 + i as f64 * step_x;
                (x, items[i].0.clone())
            })
            .collect()
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
                            x=PADDING - 22
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
                            y=height - PADDING + 14
                            text_anchor="end"
                            fill="var(--fg-muted)"
                            font_size="7"
                            transform=format!("rotate(-45 {:.1} {})", x, height - PADDING + 14)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_label_indices_empty() {
        assert!(select_label_indices(0, 6).is_empty());
    }

    #[test]
    fn select_label_indices_all_when_under_max() {
        assert_eq!(select_label_indices(4, 6), vec![0, 1, 2, 3]);
    }

    #[test]
    fn select_label_indices_thins_twenty_to_ten_plus_last() {
        // step = ceil(20/10) = 2 → [0,2,...,18], then append last (19)
        assert_eq!(
            select_label_indices(20, 10),
            vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 19],
        );
    }

    #[test]
    fn select_label_indices_always_includes_last() {
        let indices = select_label_indices(50, 6);
        assert_eq!(indices.last(), Some(&49));
        assert!(indices.len() <= 7);
    }

    #[test]
    fn select_label_indices_starts_at_zero() {
        let indices = select_label_indices(50, 6);
        assert_eq!(indices.first(), Some(&0));
        assert!(indices.len() <= 7);
    }
}
