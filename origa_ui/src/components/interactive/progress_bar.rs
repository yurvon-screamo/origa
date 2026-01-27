use leptos::prelude::*;

#[component]
pub fn ProgressBar(
    #[prop(into, optional)] current: Option<u32>,
    #[prop(into, optional)] total: Option<u32>,
    #[prop(into, optional)] percentage: Option<f32>,
    #[prop(into, optional)] show_text: Option<bool>,
    #[prop(into, optional)] show_percentage: Option<bool>,
    #[prop(into, optional)] color: Option<&'static str>,
) -> impl IntoView {
    let show_text_label = show_text.unwrap_or(true);
    let show_percent_label = show_percentage.unwrap_or(false);
    let progress_color = color.unwrap_or("#4a6fa5");

    // Calculate progress percentage
    let progress_percent = Signal::derive(move || {
        if let Some(p) = percentage {
            p
        } else if let (Some(c), Some(t)) = (current, total) {
            if t == 0 {
                0.0
            } else {
                (c as f32 / t as f32) * 100.0
            }
        } else {
            0.0
        }
    });

    // Calculate text display
    let progress_text = Signal::derive(move || {
        if let (Some(c), Some(t)) = (current, total) {
            format!("{} / {}", c, t)
        } else if let Some(p) = percentage {
            format!("{:.0}%", p)
        } else {
            "0 / 0".to_string()
        }
    });

    view! {
        <div class="progress-container">
            <Show when=move || show_text_label && (current.is_some() || percentage.is_some())>
                <div class="progress-text">
                    <span class="progress-label">Прогресс урока</span>
                    <span class="progress-value">{progress_text}</span>
                </div>
            </Show>

            <div class="progress-bar-wrapper">
                <div
                    class="progress-bar"
                    style=format!(
                        "--progress: {}%; --color: {}",
                        progress_percent.get(),
                        progress_color,
                    )
                >
                    <div class="progress-fill"></div>
                    <div class="progress-pulse"></div>
                </div>

                <Show when=move || show_percent_label>
                    <span class="progress-percentage">
                        {move || format!("{:.0}%", progress_percent.get())}
                    </span>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn CircularProgress(
    #[prop(into, optional)] size: Option<CircularSize>,
    #[prop(into, optional)] color: Option<&'static str>,
    #[prop(into, optional)] stroke_width: Option<u32>,
    #[prop(into, optional)] percentage: Option<Signal<f32>>,
) -> impl IntoView {
    let circular_size = size.unwrap_or(CircularSize::Medium);
    let progress_color = color.unwrap_or("#4a6fa5");
    let stroke = stroke_width.unwrap_or(4);
    let progress_percent = percentage.unwrap_or_else(|| Signal::derive(|| 0.0));

    let (size_class, size_pixels) = match circular_size {
        CircularSize::Small => ("circular-small", 60),
        CircularSize::Medium => ("circular-medium", 80),
        CircularSize::Large => ("circular-large", 120),
    };

    // Calculate stroke-dashoffset based on percentage (0-100%)
    // Full circle circumference: 2 * PI * 54 ≈ 339.292
    // At 0%: offset = 339.292 (full circle hidden)
    // At 100%: offset = 0 (full circle visible)
    let circumference = 339.292;
    let dash_offset = Signal::derive(move || {
        let percent = progress_percent.get().clamp(0.0, 100.0);
        circumference * (1.0 - percent / 100.0)
    });

    view! {
        <div class=format!("circular-progress {}", size_class)>
            <svg width=size_pixels height=size_pixels viewBox="0 0 120 120" class="progress-svg">
                <circle cx="60" cy="60" r="54" fill="none" stroke="#e0ddd6" stroke-width=stroke />

                <circle
                    cx="60"
                    cy="60"
                    r="54"
                    fill="none"
                    stroke=progress_color
                    stroke-width=stroke
                    stroke-linecap="round"
                    stroke-dasharray="339.292"
                    stroke-dashoffset=move || dash_offset.get()
                    class="progress-circle"
                    style=move || {
                        format!(
                            "--progress: {}%; --color: {}",
                            progress_percent.get(),
                            progress_color,
                        )
                    }
                />
            </svg>

            <div class="circular-text">{move || format!("{:.0}%", progress_percent.get())}</div>
        </div>
    }
}

#[component]
pub fn StepIndicator(
    current: Signal<Option<usize>>,
    total: u32,
    active: Signal<bool>,
) -> impl IntoView {
    let total_steps = total;

    view! {
        <div class="step-indicator">
            <div class="step-info">
                <span class="step-text">Шаг</span>
                <span class="step-current">
                    {move || { current.get().map(|v| v + 1).unwrap_or(0) }}
                </span>
                <span class="step-total">" из "{total_steps}</span>
            </div>

            <div class="step-dots">
                {(1..=total_steps.min(10))
                    .map(move |step| {
                        let current_step = current.get().map(|v| v + 1).unwrap_or(0) as u32;
                        let is_completed = step <= current_step;
                        let is_current_step = step == current_step;
                        view! {
                            <div
                                class=move || {
                                    format!(
                                        "step-dot {} {}",
                                        if is_completed { "completed" } else { "" },
                                        if is_current_step && active.get() { "active" } else { "" },
                                    )
                                }
                                title=format!("Шаг {} из {}", step, total_steps)
                            ></div>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum CircularSize {
    Small,
    #[default]
    Medium,
    Large,
}
