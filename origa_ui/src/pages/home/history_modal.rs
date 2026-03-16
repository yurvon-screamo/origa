use crate::ui_components::{LineChart, Modal, Text, TextSize, TypographyVariant};
use chrono::TimeZone;
use leptos::prelude::*;
use origa::domain::DailyHistoryItem;

#[derive(Clone, Copy, PartialEq)]
pub enum StatMetric {
    TotalCards,
    Learned,
    InProgress,
    New,
    HighDifficulty,
}

impl StatMetric {
    pub fn title(&self) -> &'static str {
        match self {
            StatMetric::TotalCards => "Всего карточек",
            StatMetric::Learned => "Изучено",
            StatMetric::InProgress => "В процессе",
            StatMetric::New => "Новые",
            StatMetric::HighDifficulty => "Сложные",
        }
    }
}

fn get_metric_value(item: &DailyHistoryItem, metric: StatMetric) -> usize {
    match metric {
        StatMetric::TotalCards => item.total_words(),
        StatMetric::Learned => item.known_words(),
        StatMetric::InProgress => item.in_progress_words(),
        StatMetric::New => item.new_words(),
        StatMetric::HighDifficulty => item.high_difficulty_words(),
    }
}

fn format_date(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    let local = chrono::Local.from_utc_datetime(&timestamp.naive_utc());
    local.format("%d %b").to_string()
}

#[component]
pub fn HistoryModal(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(into)] metric: Signal<StatMetric>,
    #[prop(into)] history: Signal<Vec<DailyHistoryItem>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let is_open_rw = RwSignal::new(is_open.get_untracked());

    Effect::new(move || {
        is_open_rw.set(is_open.get());
    });

    let recent_history = move || {
        let mut items: Vec<_> = history.get();
        items.sort_by_key(|a| a.timestamp());
        let start = items.len().saturating_sub(7);
        items.into_iter().skip(start).collect::<Vec<_>>()
    };

    let has_data = move || !recent_history().is_empty();

    let chart_data = Signal::derive(move || {
        recent_history()
            .into_iter()
            .map(|item| {
                let date_str = format_date(item.timestamp());
                let value = get_metric_value(&item, metric.get()) as f64;
                (date_str, value)
            })
            .collect::<Vec<_>>()
    });

    let _on_close_click = Callback::new(move |_: leptos::ev::MouseEvent| {
        on_close.run(());
    });

    let title = move || metric.get().title().to_string();

    view! {
        <Modal
            is_open=is_open_rw
            title=Signal::derive(title)
        >
            <div class="space-y-4">
                {move || if has_data() {
                    view! {
                        <div class="flex justify-center">
                            <LineChart
                                data=chart_data
                                width=380
                                height=180
                            />
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <Text
                            size=TextSize::Default
                            variant=TypographyVariant::Muted
                            class=Signal::derive(|| "text-center py-8".to_string())
                        >
                            "Нет данных для отображения"
                        </Text>
                    }.into_any()
                }}
            </div>
        </Modal>
    }
}
