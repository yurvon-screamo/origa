use crate::components::cards::base_card::BaseCard;
use crate::components::cards::vocab_card::StatusBadge;
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

#[component]
pub fn GrammarCard(
    card: GrammarCardData,
    #[prop(into, optional)] on_add: Option<Callback<String>>,
    #[prop(into, optional)] on_remove: Option<Callback<String>>,
    #[prop(into, optional)] on_tap: Option<Callback<String>>,
) -> impl IntoView {
    let card_id_add = card.id.clone();
    let handle_add = move |_| {
        if let Some(handler) = on_add {
            handler.run(card_id_add.clone());
        }
    };

    let card_id_remove = card.id.clone();
    let handle_remove = move |_| {
        if let Some(handler) = on_remove {
            handler.run(card_id_remove.clone());
        }
    };

    let handle_tap = Callback::new(move |_ev: leptos::ev::MouseEvent| {
        if let Some(handler) = on_tap {
            handler.run(card.id.clone());
        }
    });

    let jlpt_color = get_jlpt_color(&card.jlpt_level);
    let is_added = card.is_in_knowledge_set;

    view! {
        <BaseCard
            class=format!(
                "grammar-card {} {}",
                if is_added { "grammar-added" } else { "grammar-browse" },
                if !is_added { "hover-scale" } else { "" },
            )
            onclick=handle_tap
        >
            <div class="grammar-header">
                <div class="grammar-pattern">
                    <span class="pattern-text">{card.pattern}</span>
                    <div
                        class="jlpt-badge"
                        style=format!("background: {}; color: white;", jlpt_color)
                    >
                        {card.jlpt_level.to_string()}
                    </div>
                </div>
                <StatusBadge status=card.status />
            </div>

            <div class="grammar-meaning">
                <h4 class="meaning-title">Значение:</h4>
                <p class="meaning-text">{card.meaning}</p>
            </div>

            <div class="grammar-details">
                <div class="detail-section">
                    <h5 class="detail-heading">Присоединение:</h5>
                    <p class="detail-text">{card.attachment_rules}</p>
                </div>

                <div class="detail-section">
                    <h5 class="detail-heading">Уровень сложности:</h5>
                    <div class="difficulty-display">
                        <span class="difficulty-stars">
                            {render_difficulty_stars(card.difficulty)}
                        </span>
                        <span class="difficulty-text">{card.difficulty_text}</span>
                    </div>
                </div>

                <div class="detail-section">
                    <h5 class="detail-heading">Стабильность:</h5>
                    <div class="difficulty-display">
                        <div
                            class="progress-bar stability"
                            style=format!(
                                "--progress: {}%; --color: {}",
                                card.stability,
                                get_stability_color(card.stability)
                            )
                        >
                            <div class="progress-fill"></div>
                        </div>
                        <span class="difficulty-text">{format!("{}%", card.stability)}</span>
                    </div>
                </div>
            </div>

            {is_added
                .then(|| {
                    view! {
                        <div class="grammar-progress">
                            <div class="progress-item">
                                <span class="progress-label">
                                    Следующее повторение:
                                </span>
                                <span class="progress-date">{format_date(card.next_review)}</span>
                            </div>
                        </div>
                    }
                })}

            <div class="card-actions">
                {is_added
                    .then(|| {
                        let handle_remove_local = handle_remove;
                        view! {
                            <button
                                class="icon-button remove-btn"
                                on:click=handle_remove_local
                                aria-label="Удалить из изучения"
                            >
                                {"➖"}
                            </button>
                        }
                    })}
                {(!is_added)
                    .then(|| {
                        let handle_add_local = handle_add;
                        view! {
                            <button class="button button-primary add-btn" on:click=handle_add_local>
                                "+ Добавить"
                            </button>
                        }
                    })}
            </div>
        </BaseCard>
    }
}

fn get_jlpt_color(level: &JapaneseLevel) -> &'static str {
    match level {
        JapaneseLevel::N5 => "#4a6fa5",
        JapaneseLevel::N4 => "#5a8c5a",
        JapaneseLevel::N3 => "#b08d57",
        JapaneseLevel::N2 => "#b85450",
        JapaneseLevel::N1 => "#8b4a6f",
    }
}

#[derive(Clone)]
pub struct GrammarCardData {
    pub id: String,
    pub pattern: String,
    pub meaning: String,
    pub attachment_rules: String,
    pub difficulty: u32,
    pub difficulty_text: String,
    pub stability: u32,
    pub jlpt_level: JapaneseLevel,
    pub status: crate::components::cards::vocab_card::CardStatus,
    pub next_review: chrono::NaiveDateTime,
    pub is_in_knowledge_set: bool,
}

fn render_difficulty_stars(difficulty: u32) -> String {
    let stars = match difficulty {
        0..=20 => "⭐",
        21..=40 => "⭐⭐",
        41..=60 => "⭐⭐⭐",
        61..=80 => "⭐⭐⭐⭐",
        _ => "⭐⭐⭐⭐⭐",
    };
    stars.to_string()
}

fn get_stability_color(stability: u32) -> &'static str {
    match stability {
        0..=20 => "#b85450",   // Red - Low
        21..=40 => "#b08d57",  // Yellow
        41..=60 => "#4a6fa5",  // Blue
        61..=80 => "#66a182",  // Light green
        81..=100 => "#5a8c5a", // Green - High
        _ => "#666666",        // Gray
    }
}

fn format_date(date: chrono::NaiveDateTime) -> String {
    let today = chrono::Local::now().date_naive();
    let days_diff = (date.date() - today).num_days();

    match days_diff {
        0 => "Сегодня".to_string(),
        1 => "Завтра".to_string(),
        2..=7 => format!("Через {} дня", days_diff),
        8..=30 => format!("Через {} дней", days_diff),
        31..=365 => format!("Через {} месяцев", days_diff / 30),
        _ => format!("{}", date.format("%d.%m.%Y")),
    }
}
