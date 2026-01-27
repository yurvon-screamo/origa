use leptos::prelude::*;
use crate::components::cards::base_card::BaseCard;
use crate::components::cards::vocab_card::StatusBadge;
use crate::components::forms::jlpt_level_filter::JlptLevel;

#[component]
pub fn GrammarCard(
    card: GrammarCardData,
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
            class=format!("grammar-card {} {}", 
                if is_added { "grammar-added" } else { "grammar-browse" },
                if !is_added { "hover-scale" } else { "" }
            )
            onclick=Some(handle_tap)
        >
            <div class="grammar-header">
                <div class="grammar-pattern">
                    <span class="pattern-text">{card.pattern}</span>
                    <div class="jlpt-badge" style=format!("background: {}; color: white;", jlpt_color)>
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
                        <span class="difficulty-stars">{render_difficulty_stars(card.difficulty)}</span>
                        <span class="difficulty-text">{card.difficulty_text}</span>
                    </div>
                </div>
            </div>
            
            <div class="grammar-examples">
                <h5 class="examples-heading">Примеры использования:</h5>
                <div class="examples-list">
                    {card.examples.iter().enumerate().map(|(i, example)| view! {
                        <div class="example-item">
                            <div class="example-number">{i + 1}.</div>
                            <div class="example-content">
                                <div class="example-japanese">
                                    <span class="example-grammar">{example.grammar}</span>
                                    <span class="example-sentence">{example.sentence}</span>
                                </div>
                                <div class="example-translation">{example.translation}</div>
                                <div class="example-romaji">{example.romaji}</div>
                            </div>
                        </div>
                    }).collect_view()}
                </div>
            </div>
            
            {if is_added {
                view! {
                    <div class="grammar-progress">
                        <div class="progress-item">
                            <span class="progress-label">Следующее повторение:</span>
                            <span class="progress-date">{format_date(card.next_review)}</span>
                        </div>
                    </div>
                }
            }}
            
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
pub struct GrammarCardData {
    pub id: String,
    pub pattern: String,
    pub meaning: String,
    pub attachment_rules: String,
    pub difficulty: u32,
    pub difficulty_text: String,
    pub jlpt_level: JlptLevel,
    pub examples: Vec<GrammarExample>,
    pub status: crate::components::cards::vocab_card::CardStatus,
    pub next_review: chrono::NaiveDate,
    pub is_in_knowledge_set: bool,
}

#[derive(Clone)]
pub struct GrammarExample {
    pub grammar: String,
    pub sentence: String,
    pub translation: String,
    pub romaji: String,
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