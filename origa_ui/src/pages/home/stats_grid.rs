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
    open_history: impl Fn(StatMetric) -> Callback<()> + 'static,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-6">
            <LessonButtonsCard />

            <StatCard
                title=Signal::derive(|| "Всего карточек".to_string())
                value=total_cards
                subtitle=Signal::derive(|| "в базе".to_string())
                delta=total_cards_delta
                on_history=open_history(StatMetric::TotalCards)
            />

            <StatCard
                title=Signal::derive(|| "Изучено".to_string())
                value=learned
                subtitle=Signal::derive(|| "карточек".to_string())
                delta=learned_delta
                on_history=open_history(StatMetric::Learned)
            />

            <StatCard
                title=Signal::derive(|| "В процессе".to_string())
                value=in_progress
                subtitle=Signal::derive(|| "изучения".to_string())
                delta=in_progress_delta
                on_history=open_history(StatMetric::InProgress)
            />

            <StatCard
                title=Signal::derive(|| "Новые".to_string())
                value=new_cards
                subtitle=Signal::derive(|| "карточек".to_string())
                delta=new_delta
                on_history=open_history(StatMetric::New)
            />

            <StatCard
                title=Signal::derive(|| "Сложные".to_string())
                value=high_difficulty
                subtitle=Signal::derive(|| "требуют внимания".to_string())
                delta=high_difficulty_delta
                on_history=open_history(StatMetric::HighDifficulty)
            />
        </div>
    }
}
