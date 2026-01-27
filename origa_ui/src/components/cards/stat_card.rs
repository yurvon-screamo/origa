use leptos::prelude::*;

#[component]
pub fn StatCard(
    title: String,
    value: String,
    #[prop(optional)] trend: Option<String>,
    #[prop(optional)] show_history: Option<bool>,
    #[prop(optional)] highlight: Option<bool>,
    #[prop(optional)] on_history_click: Option<Callback<()>>,
) -> impl IntoView {
    let is_highlighted = highlight.unwrap_or(false);
    let has_trend = trend.is_some();
    let show_history_btn = show_history.unwrap_or(false);
    
    let handle_history_click = move |_| {
        if let Some(handler) = on_history_click {
            handler.run(());
        }
    };
    
    view! {
        <BaseCard class=format!(
            "stat-card {} {}",
            if is_highlighted { "stat-card-highlighted" } else { "" },
            if has_trend { "has-trend" } else { "" }
        )>
            <div class="stat-content">
                <div class="stat-header">
                    <h3 class="stat-title">{title}</h3>
                    {show_history_btn.then(|| view! {
                        <button 
                            class="icon-button stat-history-btn"
                            on:click=handle_history_click
                            aria-label="История"
                        >
                            "📈"
                        </button>
                    })}
                </div>
                
                <div class="stat-value">{value}</div>
                
                {trend.map(|trend_val| view! {
                    <div class="stat-trend">
                        <span class="trend-indicator">
                            {if trend_val.starts_with('+') { "📈" } 
                             else if trend_val.starts_with('-') { "📉" } 
                             else { "➡️" }}
                        </span>
                        <span class="trend-value">{trend_val}</span>
                    </div>
                })}
            </div>
        </BaseCard>
    }
}

#[component]
pub fn StudyButton(
    button_type: StudyButtonType,
    #[prop(optional)] count: Option<u32>,
    #[prop(optional)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    let (title, subtitle, icon, color_class) = match button_type {
        StudyButtonType::Lesson => (
            "Урок",
            count.map(|c| format!("{} новых карточек", c)).unwrap_or_else(|| "Начать изучение".to_string()),
            "📚",
            "button-primary"
        ),
        StudyButtonType::Fixation => (
            "Закрепление",
            count.map(|c| format!("{} карточек к повторению", c)).unwrap_or_else(|| "Повторить".to_string()),
            "🔄",
            "button-secondary"
        ),
    };
    
    let handle_click = move |_| {
        if let Some(handler) = on_click {
            handler.run(());
        }
    };
    
    view! {
        <button 
            class=format!("button button-large study-button {}", color_class)
            on:click=handle_click
        >
            <span class="study-button-icon">{icon}</span>
            <div class="study-button-content">
                <div class="study-button-title">{title}</div>
                <div class="study-button-subtitle">{subtitle}</div>
            </div>
        </button>
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum StudyButtonType {
    Lesson,
    Fixation,
}