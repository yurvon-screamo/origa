use crate::components::cards::base_card::BaseCard;
use leptos::prelude::*;

#[component]
pub fn StatCard(
    title: String,
    #[prop(into)] value: Signal<String>,
    #[prop(optional)] trend: Option<String>,
    #[prop(optional)] show_history: Option<bool>,
    #[prop(optional)] highlight: Option<bool>,
    #[prop(optional)] on_history_click: Option<Callback<()>>,
) -> impl IntoView {
    let is_highlighted = highlight.unwrap_or(false);
    let has_trend = trend.is_some();
    let show_history_btn = show_history.unwrap_or(false);

    view! {
        <BaseCard class=format!(
            "stat-card {} {}",
            if is_highlighted { "stat-card-highlighted" } else { "" },
            if has_trend { "has-trend" } else { "" },
        )>
            <div class="stat-content">
                <div class="stat-header">
                    <h3 class="stat-title">{title}</h3>
                    {show_history_btn
                        .then(|| {
                            let on_history_click_local = on_history_click;
                            let handle_click = move |_| {
                                if let Some(handler) = on_history_click_local {
                                    handler.run(());
                                }
                            };
                            view! {
                                <button
                                    class="icon-button stat-history-btn"
                                    on:click=handle_click
                                    aria-label="История"
                                >
                                    {"📈"}
                                </button>
                            }
                        })}
                </div>

                <div class="stat-value">{move || value.get()}</div>

                {trend
                    .map(|trend_val| {
                        view! {
                            <div class="stat-trend">
                                <span class="trend-indicator">
                                    {if trend_val.starts_with('+') {
                                        "📈"
                                    } else if trend_val.starts_with('-') {
                                        "📉"
                                    } else {
                                        "➡"
                                    }}
                                </span>
                                <span class="trend-value">{trend_val}</span>
                            </div>
                        }
                    })}
            </div>
        </BaseCard>
    }
}

#[component]
pub fn StudyButton(
    button_type: StudyButtonType,
    #[prop(into, optional)] count: Option<Signal<u32>>,
    #[prop(optional)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    let count_clone = count;
    let subtitle = Signal::derive(move || {
        let count_val = count_clone.as_ref().map(|c| c.get()).unwrap_or(0);
        match button_type {
            StudyButtonType::Lesson => {
                if count_val > 0 {
                    format!("{} новых карточек", count_val)
                } else {
                    "Начать изучение".to_string()
                }
            }
            StudyButtonType::Fixation => {
                if count_val > 0 {
                    format!("{} карточек к повторению", count_val)
                } else {
                    "Повторить".to_string()
                }
            }
        }
    });
    let (title, icon, color_class) = match button_type {
        StudyButtonType::Lesson => ("Урок", "📚", "button-primary"),
        StudyButtonType::Fixation => ("Закрепление", "🔄", "button-secondary"),
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
                <div class="study-button-subtitle">{move || subtitle.get()}</div>
            </div>
        </button>
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum StudyButtonType {
    Lesson,
    Fixation,
}
