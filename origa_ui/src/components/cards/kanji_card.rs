use leptos::prelude::*;
use crate::components::cards::base_card::BaseCard;
use crate::components::cards::vocab_card::StatusBadge;

#[component]
pub fn KanjiCard(
    card: KanjiCardData,
    #[prop(into, optional)] on_add: Option<Callback<String>>,
    #[prop(into, optional)] on_remove: Option<Callback<String>>,
    #[prop(into, optional)] on_tap: Option<Callback<String>>,
) -> impl IntoView {
    let handle_add = move |_| {
        if let Some(handler) = on_add {
            handler.run(card.id.clone());
        }
    };
    
    let handle_remove = move |_| {
        if let Some(handler) = on_remove {
            handler.run(card.id.clone());
        }
    };
    
    let handle_tap = move |_| {
        if let Some(handler) = on_tap {
            handler.run(card.id.clone());
        }
    };
    
    let jlpt_color = card.jlpt_level.difficulty_color();
    let is_added = card.is_in_knowledge_set;
    
    view! {
        <BaseCard 
            class=format!("kanji-card {} {}", 
                if is_added { "kanji-added" } else { "kanji-browse" },
                if !is_added { "hover-scale" } else { "" }
            )
            onclick=Some(handle_tap)
        >
            <div class="kanji-header">
                <div class="kanji-character-section">
                    <span class="kanji-character">{card.character}</span>
                    <span class="kanji-stroke-count">{card.stroke_count} черт</span>
                </div>
                
                <div class="kanji-meta">
                    <div class="jlpt-badge" style=format!("background: {}; color: white;", jlpt_color)>
                        {card.jlpt_level.to_string()}
                    </div>
                    <StatusBadge status=card.status />
                </div>
            </div>
            
            <div class="kanji-meanings">
                <h4 class="meanings-title">Значения:</h4>
                <div class="meanings-list">
                    {card.meanings.iter().map(|meaning| view! {
                        <span class="meaning-item">{meaning}</span>
                    }).collect_view()}
                </div>
            </div>
            
            <div class="kanji-readings">
                <div class="reading-section">
                    <h5 class="reading-title">Onyomi:</h5>
                    <div class="readings-list onyomi">
                        {card.onyomi.iter().map(|reading| view! {
                            <span class="reading-item">{reading}</span>
                        }).collect_view()}
                    </div>
                </div>
                
                <div class="reading-section">
                    <h5 class="reading-title">Kunyomi:</h5>
                    <div class="readings-list kunyomi">
                        {card.kunyomi.iter().map(|reading| view! {
                            <span class="reading-item">{reading}</span>
                        }).collect_view()}
                    </div>
                </div>
            </div>
            
            <div class="kanji-radicals">
                <h5 class="radicals-title">Радикалы:</h5>
                <div class="radicals-list">
                    {card.radicals.iter().map(|radical| view! {
                        <span class="radical-item">
                            <span class="radical-char">{radical.character}</span>
                            <span class="radical-meaning">{radical.meaning}</span>
                        </span>
                    }).collect_view()}
                </div>
            </div>
            
            <div class="kanji-progress">
                <div class="progress-item">
                    <span class="progress-label">Сложность:</span>
                    <div class="progress-bar difficulty" style=format!("--progress: {}%; --color: {}", card.difficulty, get_difficulty_color(card.difficulty))>
                        <div class="progress-fill"></div>
                    </div>
                    <span class="progress-value">{card.difficulty}%</span>
                </div>
                
                <div class="progress-item">
                    <span class="progress-label">Стабильность:</span>
                    <div class="progress-bar stability" style=format!("--progress: {}%; --color: {}", card.stability, get_stability_color(card.stability))>
                        <div class="progress-fill"></div>
                    </div>
                    <span class="progress-value">{card.stability}%</span>
                </div>
                
                {is_added.then(|| view! {
                    <div class="next-review">
                        <span class="review-label">Следующее повторение:</span>
                        <span class="review-date">{format_date(card.next_review)}</span>
                    </div>
                })}
            </div>
            
            <div class="card-actions">
                {if is_added {
                    view! {
                        <button class="icon-button remove-btn" on:click=handle_remove aria-label="Удалить из изучения">
                            "➖"
                        </button>
                    }
                } else {
                    view! {
                        <button class="button button-primary add-btn" on:click=handle_add>
                            "+ Добавить"
                        </button>
                    }
                }}
            </div>
        </BaseCard>
    }
}

#[derive(Clone)]
pub struct KanjiCardData {
    pub id: String,
    pub character: String,
    pub stroke_count: u8,
    pub jlpt_level: crate::components::forms::jlpt_level_filter::JlptLevel,
    pub meanings: Vec<String>,
    pub onyomi: Vec<String>,
    pub kunyomi: Vec<String>,
    pub radicals: Vec<RadicalInfo>,
    pub status: crate::components::cards::vocab_card::CardStatus,
    pub difficulty: u32,
    pub stability: u32,
    pub next_review: chrono::NaiveDate,
    pub is_in_knowledge_set: bool,
}

#[derive(Clone)]
pub struct RadicalInfo {
    pub character: String,
    pub meaning: String,
    pub strokes: u8,
}

fn get_difficulty_color(difficulty: u32) -> &'static str {
    match difficulty {
        0..=20 => "#5a8c5a",        // Green - Easy
        21..=40 => "#66a182",       // Light green
        41..=60 => "#b08d57",       // Yellow - Medium
        61..=80 => "#b85450",       // Light red
        81..=100 => "#8b2635",      // Dark red - Hard
        _ => "#666666",             // Gray
    }
}

fn get_stability_color(stability: u32) -> &'static str {
    match stability {
        0..=20 => "#b85450",        // Red - Low
        21..=40 => "#b08d57",       // Yellow
        41..=60 => "#4a6fa5",       // Blue
        61..=80 => "#66a182",       // Light green
        81..=100 => "#5a8c5a",      // Green - High
        _ => "#666666",             // Gray
    }
}

fn format_date(date: chrono::NaiveDate) -> String {
    let today = chrono::Local::now().date_naive();
    let days_diff = (date - today).num_days();
    
    match days_diff {
        0 => "Сегодня".to_string(),
        1 => "Завтра".to_string(),
        2..=7 => format!("Через {} дня", days_diff),
        8..=30 => format!("Через {} дней", days_diff),
        31..=365 => format!("Через {} месяцев", days_diff / 30),
        _ => format!("{}", date.format("%d.%m.%Y")),
    }
}