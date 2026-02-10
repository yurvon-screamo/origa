use crate::components::cards::base_card::BaseCard;
use crate::components::cards::vocab_card::StatusBadge;
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

#[component]
pub fn KanjiDetail(
    kanji: KanjiDetailData,
    #[prop(into, optional)] on_add: Option<Callback<String>>,
    #[prop(into, optional)] on_remove: Option<Callback<String>>,
    #[prop(into, optional)] on_back: Option<Callback<()>>,
) -> impl IntoView {
    let kanji_id = kanji.id.clone();
    let handle_add = move |_| {
        if let Some(handler) = on_add {
            handler.run(kanji_id.clone());
        }
    };

    let kanji_id2 = kanji.id.clone();
    let handle_remove = move |_| {
        if let Some(handler) = on_remove {
            handler.run(kanji_id2.clone());
        }
    };

    let handle_back = move |_| {
        if let Some(handler) = on_back {
            handler.run(());
        }
    };

    let is_added = kanji.is_in_knowledge_set;
    let jlpt_color = get_jlpt_color(&kanji.jlpt_level);

    view! {
        <div class="kanji-detail-page">
            // Back navigation
            <div class="detail-header">
                <button class="icon-button back-btn" on:click=handle_back>
                    "←"
                </button>
                <h1 class="detail-title">Детали кандзи</h1>
                <div class="detail-actions">
                    {is_added
                        .then(|| {
                            let handle_remove_local = handle_remove;
                            view! {
                                <button class="icon-button remove-btn" on:click=handle_remove_local>
                                    {"🗑"}
                                </button>
                            }
                        })}
                    {(!is_added)
                        .then(|| {
                            let handle_add_local = handle_add;
                            view! {
                                <button class="button button-primary" on:click=handle_add_local>
                                    "+ Добавить"
                                </button>
                            }
                        })}
                </div>
            </div>

            // Main kanji display
            <BaseCard class="kanji-main-display">
                <div class="kanji-center-section">
                    <span class="kanji-character-large">{kanji.character}</span>
                    <div class="kanji-basic-info">
                        <div
                            class="jlpt-badge-large"
                            style=format!("background: {}; color: white;", jlpt_color)
                        >
                            {kanji.jlpt_level.to_string()}
                        </div>
                    </div>
                </div>
            </BaseCard>

            // Meanings section
            <BaseCard>
                <h2 class="section-heading">Значения</h2>
                <div class="meanings-detail">
                    {kanji
                        .meanings
                        .iter()
                        .enumerate()
                        .map(|(i, meaning)| {
                            view! {
                                <div class="meaning-item">
                                    <span class="meaning-number">{i + 1}.</span>
                                    <span class="meaning-text">{meaning.clone()}</span>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </BaseCard>

            // Radicals section
            <BaseCard>
                <h2 class="section-heading">Радикалы</h2>
                <div class="radicals-detail">
                    {kanji
                        .radicals
                        .iter()
                        .map(|radical| {
                            let character = radical.character.clone();
                            let meaning = radical.meaning.clone();
                            view! {
                                <div class="radical-detail-item">
                                    <div class="radical-display">
                                        <span class="radical-char">{character}</span>
                                        <span class="radical-meaning">{meaning}</span>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </BaseCard>

            // Examples section
            <BaseCard>
                <h2 class="section-heading">Примеры использования</h2>
                <div class="examples-detail">
                    {kanji
                        .examples
                        .iter()
                        .map(|example| {
                            let kanji = example.kanji.clone();
                            let meaning = example.meaning.clone();
                            view! {
                                <div class="example-item">
                                    <div class="example-japanese">
                                        <span class="example-kanji">{kanji}</span>
                                    </div>
                                    <div class="example-meaning">{meaning}</div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </BaseCard>

            // Progress section (if in knowledge set)
            <Show when=move || is_added>
                <BaseCard>
                    <h2 class="section-heading">Прогресс изучения</h2>
                    <div class="progress-detail">
                        <div class="progress-item-detail">
                            <span class="progress-label-detail">Статус:</span>
                            <StatusBadge status=kanji.status />
                        </div>

                        <div class="progress-item-detail">
                            <span class="progress-label-detail">Сложность:</span>
                            <div
                                class="progress-bar-detailed"
                                style=format!(
                                    "--progress: {}%; --color: {}",
                                    kanji.difficulty,
                                    get_difficulty_color(kanji.difficulty),
                                )
                            >
                                <div class="progress-fill"></div>
                            </div>
                            <span class="progress-value-detail">{kanji.difficulty}%</span>
                        </div>

                        <div class="progress-item-detail">
                            <span class="progress-label-detail">Стабильность:</span>
                            <div
                                class="progress-bar-detailed"
                                style=format!(
                                    "--progress: {}%; --color: {}",
                                    kanji.stability,
                                    get_stability_color(kanji.stability),
                                )
                            >
                                <div class="progress-fill"></div>
                            </div>
                            <span class="progress-value-detail">{kanji.stability}%</span>
                        </div>

                        <div class="next-review-detail">
                            <span class="review-label-detail">
                                Следующее повторение:
                            </span>
                            <span class="review-date-detail">{format_date(kanji.next_review)}</span>
                        </div>
                    </div>
                </BaseCard>
            </Show>

        </div>
    }
}

#[derive(Clone)]
pub struct KanjiDetailData {
    pub id: String,
    pub character: String,
    pub jlpt_level: JapaneseLevel,
    pub meanings: Vec<String>,
    pub radicals: Vec<RadicalDetail>,
    pub examples: Vec<ExampleInfo>,
    pub status: crate::components::cards::vocab_card::CardStatus,
    pub difficulty: u32,
    pub stability: u32,
    pub next_review: chrono::NaiveDateTime,
    pub is_in_knowledge_set: bool,
}

#[derive(Clone)]
pub struct RadicalDetail {
    pub character: String,
    pub meaning: String,
}

#[derive(Clone)]
pub struct ExampleInfo {
    pub kanji: String,
    pub meaning: String,
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
