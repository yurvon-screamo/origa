use crate::components::cards::base_card::BaseCard;
use leptos::prelude::*;

#[component]
pub fn VocabCard(
    card: VocabularyCardData,
    #[prop(into, optional)] on_edit: Option<Callback<String>>,
    #[prop(into, optional)] on_delete: Option<Callback<String>>,
    #[prop(into, optional)] on_tap: Option<Callback<String>>,
) -> impl IntoView {
    let card_id = card.id.clone();
    let handle_edit = move |_| {
        if let Some(handler) = on_edit {
            handler.run(card_id.clone());
        }
    };

    let card_id2 = card.id.clone();
    let handle_delete = move |_| {
        if let Some(handler) = on_delete {
            handler.run(card_id2.clone());
        }
    };

    let card_id3 = card.id.clone();
    let handle_card_tap = Callback::new(move |_ev: leptos::ev::MouseEvent| {
        if let Some(handler) = on_tap {
            handler.run(card_id3.clone());
        }
    });

    let status_class = card.status.to_class();
    let difficulty_color = get_difficulty_color(card.difficulty);
    let stability_color = get_stability_color(card.stability);

    view! {
        <BaseCard
            class=format!("vocab-card {} {}", status_class, "hover-scale")
            onclick=handle_card_tap
        >
            <div class="vocab-header">
                <div class="vocab-japanese">
                    <span class="japanese-character">{card.japanese}</span>
                    <span class="japanese-reading">{card.reading}</span>
                </div>
                <StatusBadge status=card.status />
            </div>

            <div class="vocab-body">
                <div class="vocab-translation">{card.translation}</div>

                <div class="vocab-meta">
                    <div class="vocab-progress">
                        <div class="progress-item">
                            <span class="progress-label">Сложность:</span>
                            <div
                                class="progress-bar difficulty"
                                style=format!(
                                    "--progress: {}%; --color: {}",
                                    card.difficulty,
                                    difficulty_color,
                                )
                            >
                                <div class="progress-fill"></div>
                            </div>
                            <span class="progress-value">{card.difficulty}%</span>
                        </div>

                        <div class="progress-item">
                            <span class="progress-label">Стабильность:</span>
                            <div
                                class="progress-bar stability"
                                style=format!(
                                    "--progress: {}%; --color: {}",
                                    card.stability,
                                    stability_color,
                                )
                            >
                                <div class="progress-fill"></div>
                            </div>
                            <span class="progress-value">{card.stability}%</span>
                        </div>
                    </div>

                    <div class="vocab-next-review">
                        <span class="review-label">Следующее повторение:</span>
                        <span class="review-date">{format_date(card.next_review)}</span>
                    </div>
                </div>
            </div>

            <div class="card-actions">
                <button
                    class="icon-button edit-btn"
                    on:click=handle_edit
                    aria-label="Редактировать"
                >
                    "📝"
                </button>
                <button
                    class="icon-button delete-btn"
                    on:click=handle_delete
                    aria-label="Удалить"
                >
                    "🗑️"
                </button>
            </div>
        </BaseCard>
    }
}

#[component]
pub fn StatusBadge(status: CardStatus) -> impl IntoView {
    let (label, color_class) = match status {
        CardStatus::New => ("Новое", "badge-info"),
        CardStatus::InProgress => ("В процессе", "badge-warning"),
        CardStatus::Difficult => ("Сложное", "badge-error"),
        CardStatus::Mastered => ("Изучено", "badge-success"),
    };

    view! { <span class=format!("badge {}", color_class)>{label}</span> }
}

#[derive(Clone)]
pub struct VocabularyCardData {
    pub id: String,
    pub japanese: String,
    pub reading: String,
    pub translation: String,
    pub status: CardStatus,
    pub difficulty: u32,
    pub stability: u32,
    pub next_review: chrono::NaiveDateTime,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CardStatus {
    New,
    InProgress,
    Difficult,
    Mastered,
}

impl CardStatus {
    pub fn to_class(&self) -> &'static str {
        match self {
            CardStatus::New => "status-new",
            CardStatus::InProgress => "status-in-progress",
            CardStatus::Difficult => "status-difficult",
            CardStatus::Mastered => "status-mastered",
        }
    }
}

fn get_difficulty_color(difficulty: u32) -> &'static str {
    match difficulty {
        0..=20 => "#5a8c5a",   // Green - Easy
        21..=40 => "#66a182",  // Light green
        41..=60 => "#b08d57",  // Yellow - Medium
        61..=80 => "#b85450",  // Light red
        81..=100 => "#8b2635", // Dark red - Hard
        _ => "#666666",        // Gray
    }
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
