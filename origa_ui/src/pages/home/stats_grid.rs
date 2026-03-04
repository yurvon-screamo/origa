use super::{LessonButtonsCard, StatCard, StatMetric};
use leptos::prelude::*;

#[component]
pub fn StatsGrid(
    total_cards: Signal<String>,
    learned: Signal<String>,
    in_progress: Signal<String>,
    new_cards: Signal<String>,
    high_difficulty: Signal<String>,
    weekly_delta_text: Signal<String>,
    open_history: impl Fn(StatMetric) -> Callback<()> + 'static,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-6">
            <LessonButtonsCard />

            <StatCard
                title=Signal::derive(|| "Всего карточек".to_string())
                value=total_cards
                subtitle=Signal::derive(|| "в базе".to_string())
                delta=weekly_delta_text
                on_history=open_history(StatMetric::TotalCards)
            />

            <StatCard
                title=Signal::derive(|| "Изучено".to_string())
                value=learned
                subtitle=Signal::derive(|| "карточек".to_string())
                on_history=open_history(StatMetric::Learned)
            />

            <StatCard
                title=Signal::derive(|| "В процессе".to_string())
                value=in_progress
                subtitle=Signal::derive(|| "изучения".to_string())
                on_history=open_history(StatMetric::InProgress)
            />

            <StatCard
                title=Signal::derive(|| "Новые".to_string())
                value=new_cards
                subtitle=Signal::derive(|| "карточек".to_string())
                on_history=open_history(StatMetric::New)
            />

            <StatCard
                title=Signal::derive(|| "Сложные".to_string())
                value=high_difficulty
                subtitle=Signal::derive(|| "требуют внимания".to_string())
                on_history=open_history(StatMetric::HighDifficulty)
            />
        </div>
    }
}
