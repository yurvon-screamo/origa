use leptos::prelude::*;
use crate::components::cards::base_card::BaseCard;
use crate::components::cards::vocab_card::StatusBadge;
use crate::components::forms::jlpt_level_filter::JlptLevel;
use crate::components::cards::kanji_card::RadicalInfo;

#[component]
pub fn KanjiDetail(
    kanji: KanjiDetailData,
    #[prop(into, optional)] on_add: Option<Callback<String>>,
    #[prop(into, optional)] on_remove: Option<Callback<String>>,
    #[prop(into, optional)] on_back: Option<Callback<()>>,
) -> impl IntoView {
    let handle_add = move |_| {
        if let Some(handler) = on_add {
            handler.run(kanji.id.clone());
        }
    };
    
    let handle_remove = move |_| {
        if let Some(handler) = on_remove {
            handler.run(kanji.id.clone());
        }
    };
    
    let handle_back = move |_| {
        if let Some(handler) = on_back {
            handler.run(());
        }
    };
    
    let is_added = kanji.is_in_knowledge_set;
    let jlpt_color = kanji.jlpt_level.difficulty_color();
    
    view! {
        <div class="kanji-detail-page">
            // Back navigation
            <div class="detail-header">
                <button class="icon-button back-btn" on:click=handle_back>
                    "←"
                </button>
                <h1 class="detail-title">Детали кандзи</h1>
                <div class="detail-actions">
                    {if is_added {
                        view! {
                            <button class="icon-button remove-btn" on:click=handle_remove>
                                "🗑️"
                            </button>
                        }
                    } else {
                        view! {
                            <button class="button button-primary" on:click=handle_add>
                                "+ Добавить"
                            </button>
                        }
                    }}
                </div>
            </div>
            
            // Main kanji display
            <BaseCard class="kanji-main-display">
                <div class="kanji-center-section">
                    <span class="kanji-character-large">{kanji.character}</span>
                    <div class="kanji-basic-info">
                        <div class="jlpt-badge-large" style=format!("background: {}; color: white;", jlpt_color)>
                            {kanji.jlpt_level.to_string()}
                        </div>
                        <div class="stroke-info">
                            <span class="stroke-count">{kanji.stroke_count} черт</span>
                            <span class="grade-level">{kanji.grade_level}</span>
                        </div>
                    </div>
                </div>
            </BaseCard>
            
            // Meanings section
            <BaseCard>
                <h2 class="section-heading">Значения</h2>
                <div class="meanings-detail">
                    {kanji.meanings.iter().enumerate().map(|(i, meaning)| view! {
                        <div class="meaning-item">
                            <span class="meaning-number">{i + 1}.</span>
                            <span class="meaning-text">{meaning}</span>
                        </div>
                    }).collect_view()}
                </div>
            </BaseCard>
            
            // Readings section
            <div class="readings-grid">
                <BaseCard>
                    <h3 class="subsection-heading">Onyomi (китайское чтение)</h3>
                    <div class="readings-detail">
                        {kanji.onyomi.iter().enumerate().map(|(i, reading)| view! {
                            <div class="reading-detail-item">
                                <span class="reading-number">{i + 1}.</span>
                                <span class="reading-romaji">{reading.romaji}</span>
                                <span class="reading-japanese">{reading.hiragana}</span>
                            </div>
                        }).collect_view()}
                    </div>
                </BaseCard>
                
                <BaseCard>
                    <h3 class="subsection-heading">Kunyomi (японское чтение)</h3>
                    <div class="readings-detail">
                        {kanji.kunyomi.iter().enumerate().map(|(i, reading)| view! {
                            <div class="reading-detail-item">
                                <span class="reading-number">{i + 1}.</span>
                                <span class="reading-romaji">{reading.romaji}</span>
                                <span class="reading-japanese">{reading.hiragana}</span>
                                {reading.okurigana.as_ref().map(|ok| view! {
                                    <span class="reading-okurigana">{ok}</span>
                                })}
                            </div>
                        }).collect_view()}
                    </div>
                </BaseCard>
            </div>
            
            // Radicals section
            <BaseCard>
                <h2 class="section-heading">Радикалы</h2>
                <div class="radicals-detail">
                    {kanji.radicals.iter().map(|radical| view! {
                        <div class="radical-detail-item">
                            <div class="radical-display">
                                <span class="radical-char">{radical.character}</span>
                                <span class="radical-meaning">{radical.meaning}</span>
                            </div>
                            <div class="radical-meta">
                                <span class="radical-stroke">{radical.stroke_count} черт</span>
                                <span class="radical-position">{radical.position}</span>
                            </div>
                        </div>
                    }).collect_view()}
                </div>
            </BaseCard>
            
            // Examples section
            <BaseCard>
                <h2 class="section-heading">Примеры использования</h2>
                <div class="examples-detail">
                    {kanji.examples.iter().map(|example| view! {
                        <div class="example-item">
                            <div class="example-japanese">
                                <span class="example-kanji">{example.kanji}</span>
                                <span class="example-reading">{example.reading}</span>
                            </div>
                            <div class="example-meaning">
                                {example.meaning}
                            </div>
                            <div class="example-romaji">
                                {example.romaji}
                            </div>
                        </div>
                    }).collect_view()}
                </div>
            </BaseCard>
            
            // Progress section (if in knowledge set)
            {if is_added {
                view! {
                    <BaseCard>
                        <h2 class="section-heading">Прогресс изучения</h2>
                        <div class="progress-detail">
                            <div class="progress-item-detail">
                                <span class="progress-label-detail">Статус:</span>
                                <StatusBadge status=kanji.status />
                            </div>
                            
                            <div class="progress-item-detail">
                                <span class="progress-label-detail">Сложность:</span>
                                <div class="progress-bar-detailed" style=format!("--progress: {}%; --color: {}", kanji.difficulty, get_difficulty_color(kanji.difficulty))>
                                    <div class="progress-fill"></div>
                                </div>
                                <span class="progress-value-detail">{kanji.difficulty}%</span>
                            </div>
                            
                            <div class="progress-item-detail">
                                <span class="progress-label-detail">Стабильность:</span>
                                <div class="progress-bar-detailed" style=format!("--progress: {}%; --color: {}", kanji.stability, get_stability_color(kanji.stability))>
                                    <div class="progress-fill"></div>
                                </div>
                                <span class="progress-value-detail">{kanji.stability}%</span>
                            </div>
                            
                            <div class="next-review-detail">
                                <span class="review-label-detail">Следующее повторение:</span>
                                <span class="review-date-detail">{format_date(kanji.next_review)}</span>
                            </div>
                        </div>
                    </BaseCard>
                }
            }}
            
            // Study hints
            <BaseCard>
                <h2 class="section-heading">Советы по изучению</h2>
                <div class="study-hints">
                    <div class="hint-item">
                        <span class="hint-icon">💡</span>
                        <div class="hint-content">
                            <h4 class="hint-title">Мнемоника</h4>
                            <p class="hint-text">{kanji.mnemonic_hint}</p>
                        </div>
                    </div>
                    
                    <div class="hint-item">
                        <span class="hint-icon">✏️</span>
                        <div class="hint-content">
                            <h4 class="hint-title">Порядок черт</h4>
                            <p class="hint-text">{kanji.stroke_order_hint}</p>
                        </div>
                    </div>
                    
                    <div class="hint-item">
                        <span class="hint-icon">🔗</span>
                        <div class="hint-content">
                            <h4 class="hint-title">Связанные кандзи</h4>
                            <div class="related-kanji">
                                {kanji.related_kanji.iter().map(|related| view! {
                                    <span class="related-char">{related}</span>
                                }).collect_view()}
                            </div>
                        </div>
                    </div>
                </div>
            </BaseCard>
        </div>
    }
}

#[derive(Clone)]
pub struct KanjiDetailData {
    pub id: String,
    pub character: String,
    pub stroke_count: u8,
    pub grade_level: String,
    pub jlpt_level: JlptLevel,
    pub meanings: Vec<String>,
    pub onyomi: Vec<ReadingInfo>,
    pub kunyomi: Vec<ReadingInfo>,
    pub radicals: Vec<RadicalDetail>,
    pub examples: Vec<ExampleInfo>,
    pub status: crate::components::cards::vocab_card::CardStatus,
    pub difficulty: u32,
    pub stability: u32,
    pub next_review: chrono::NaiveDate,
    pub is_in_knowledge_set: bool,
    pub mnemonic_hint: String,
    pub stroke_order_hint: String,
    pub related_kanji: Vec<String>,
}

#[derive(Clone)]
pub struct ReadingInfo {
    pub romaji: String,
    pub hiragana: String,
    pub okurigana: Option<String>,
}

#[derive(Clone)]
pub struct RadicalDetail {
    pub character: String,
    pub meaning: String,
    pub stroke_count: u8,
    pub position: String,
}

#[derive(Clone)]
pub struct ExampleInfo {
    pub kanji: String,
    pub reading: String,
    pub meaning: String,
    pub romaji: String,
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