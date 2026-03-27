use super::{LessonButtonsCard, StatCard, StatMetric};
use leptos::prelude::*;

#[component]
pub fn StatsGrid(
    total_cards: Signal<String>,
    total_cards_delta: Signal<String>,
    learned: Signal<String>,
    learned_delta: Signal<String>,
    in_progress: Signal<String>,
    in_progress_delta: Signal<String>,
    new_cards: Signal<String>,
    new_delta: Signal<String>,
    high_difficulty: Signal<String>,
    high_difficulty_delta: Signal<String>,
    positive: Signal<String>,
    positive_delta: Signal<String>,
    negative: Signal<String>,
    negative_delta: Signal<String>,
    total_ratings: Signal<String>,
    total_ratings_delta: Signal<String>,
    open_history: impl Fn(StatMetric) -> Callback<()> + 'static,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    view! {
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 2xl:grid-cols-9 gap-6" data-testid=test_id_val>
            <LessonButtonsCard test_id=Signal::derive(|| "lesson-buttons".to_string()) />

            <StatCard
                title=Signal::derive(|| "Всего карточек".to_string())
                value=total_cards
                subtitle=Signal::derive(|| "в базе".to_string())
                delta=total_cards_delta
                on_history=open_history(StatMetric::TotalCards)
                test_id=Signal::derive(|| "stat-total-cards".to_string())
            />

            <StatCard
                title=Signal::derive(|| "Изучено".to_string())
                value=learned
                subtitle=Signal::derive(|| "карточек".to_string())
                delta=learned_delta
                on_history=open_history(StatMetric::Learned)
                test_id=Signal::derive(|| "stat-learned".to_string())
            />

            <StatCard
                title=Signal::derive(|| "В процессе".to_string())
                value=in_progress
                subtitle=Signal::derive(|| "изучения".to_string())
                delta=in_progress_delta
                on_history=open_history(StatMetric::InProgress)
                test_id=Signal::derive(|| "stat-in-progress".to_string())
            />

            <StatCard
                title=Signal::derive(|| "Новые".to_string())
                value=new_cards
                subtitle=Signal::derive(|| "карточек".to_string())
                delta=new_delta
                on_history=open_history(StatMetric::New)
                test_id=Signal::derive(|| "stat-new".to_string())
            />

            <StatCard
                title=Signal::derive(|| "Сложные".to_string())
                value=high_difficulty
                subtitle=Signal::derive(|| "требуют внимания".to_string())
                delta=high_difficulty_delta
                on_history=open_history(StatMetric::HighDifficulty)
                test_id=Signal::derive(|| "stat-high-difficulty".to_string())
            />

            <StatCard
                title=Signal::derive(|| "Позитивные оценки".to_string())
                value=positive
                subtitle=Signal::derive(|| "оценок".to_string())
                delta=positive_delta
                on_history=open_history(StatMetric::PositiveRatings)
                test_id=Signal::derive(|| "stat-positive".to_string())
            />

            <StatCard
                title=Signal::derive(|| "Негативные оценки".to_string())
                value=negative
                subtitle=Signal::derive(|| "оценок".to_string())
                delta=negative_delta
                on_history=open_history(StatMetric::NegativeRatings)
                test_id=Signal::derive(|| "stat-negative".to_string())
            />

            <StatCard
                title=Signal::derive(|| "Всего оценок".to_string())
                value=total_ratings
                subtitle=Signal::derive(|| "за сегодня".to_string())
                delta=total_ratings_delta
                on_history=open_history(StatMetric::TotalRatings)
                test_id=Signal::derive(|| "stat-total-ratings".to_string())
            />
        </div>
    }
}
