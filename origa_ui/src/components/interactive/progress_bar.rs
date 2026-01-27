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
        } else if let ((Some(c), Some(t))) = (current, total) {
            if t == 0 { 0.0 } else { (c as f32 / t as f32) * 100.0 }
        } else {
            0.0
        }
    });
    
    // Calculate text display
    let progress_text = Signal::derive(move || {
        if let ((Some(c), Some(t))) = (current, total) {
            format!("{} / {}", c, t)
        } else if let Some(p) = percentage {
            format!("{:.0}%", p)
        } else {
            "0 / 0".to_string()
        }
    });
    
    view! {
        <div class="progress-container">
            <Show when=move || show_text_label && (current().is_some() || percentage().is_some())>
                <div class="progress-text">
                    <span class="progress-label">Прогресс урока</span>
                    <span class="progress-value">{progress_text}</span>
                </div>
            </Show>
            
            <div class="progress-bar-wrapper">
                <div 
                    class="progress-bar"
                    style=format!("--progress: {}%; --color: {}", progress_percent(), progress_color)
                >
                    <div class="progress-fill"></div>
                    <div class="progress-pulse"></div>
                </div>
                
                <Show when=show_percent_label>
                    <span class="progress-percentage">
                        {move || format!("{:.0}%", progress_percent())}
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
) -> impl IntoView {
    let circular_size = size.unwrap_or(CircularSize::Medium);
    let progress_color = color.unwrap_or("#4a6fa5");
    let stroke = stroke_width.unwrap_or(4);
    
    let (size_class, size_pixels) = match circular_size {
        CircularSize::Small => ("circular-small", 60),
        CircularSize::Medium => ("circular-medium", 80),
        CircularSize::Large => ("circular-large", 120),
    };
    
    view! {
        <div class=format!("circular-progress {}", size_class)>
            <svg 
                width=size_pixels
                height=size_pixels
                viewBox="0 0 120 120"
                class="progress-svg"
            >
                <!-- Background circle -->
                <circle
                    cx="60"
                    cy="60"
                    r="54"
                    fill="none"
                    stroke="#e0ddd6"
                    stroke-width={stroke}
                />
                
                <!-- Progress circle -->
                <circle
                    cx="60"
                    cy="60"
                    r="54"
                    fill="none"
                    stroke={progress_color}
                    stroke-width={stroke}
                    stroke-linecap="round"
                    stroke-dasharray="339.292"
                    stroke-dashoffset="169.646"
                    class="progress-circle"
                    style=format!("--progress: {}; --color: {}", "50%", progress_color)
                />
            </svg>
            
            <div class="circular-text">
                "50%"
            </div>
        </div>
    }
}

#[component]
pub fn StepIndicator(
    #[prop(into, optional)] current: Option<u32>,
    total: u32,
    #[prop(into, optional)] active: Option<bool>,
) -> impl IntoView {
    let current_step = current.unwrap_or(0);
    let is_active = active.unwrap_or(false);
    
    view! {
        <div class="step-indicator">
            <div class="step-info">
                <span class="step-text">Шаг</span>
                <span class="step-current">{current_step}</span>
                <span class="step-total">из {total}</span>
            </div>
            
            <div class="step-dots">
                {(1..=total+1).map(|step| view! {
                    <div 
                        class=format!(
                            "step-dot {}",
                            if step <= current_step { "step-completed" } else { "" },
                            if step == current_step + 1 && is_active { "step-active" } else { "" }
                        )
                    title=format!("Шаг {} из {}", step, total)
                    ></div>
                }).collect_view()}
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum CircularSize {
    Small,
    Medium,
    Large,
}

impl Default for CircularSize {
    fn default() -> Self {
        CircularSize::Medium
    }
}