use crate::i18n::{Locale, t, use_i18n};
use crate::ui_components::{LineChart, Modal, Text, TextSize, TypographyVariant};
use chrono::TimeZone;
use leptos::prelude::*;
use leptos_i18n::I18nContext;
use origa::domain::DailyHistoryItem;

#[derive(Clone, Copy, PartialEq)]
pub enum StatMetric {
    TotalCards,
    Learned,
    InProgress,
    New,
    HighDifficulty,
    PositiveRatings,
    NegativeRatings,
    TotalRatings,
}

impl StatMetric {
    pub fn title(&self, i18n: &I18nContext<Locale>) -> String {
        match self {
            StatMetric::TotalCards => i18n.get_keys().home().total_cards().inner().to_string(),
            StatMetric::Learned => i18n.get_keys().home().learned().inner().to_string(),
            StatMetric::InProgress => i18n.get_keys().home().in_progress().inner().to_string(),
            StatMetric::New => i18n.get_keys().home().new_items().inner().to_string(),
            StatMetric::HighDifficulty => i18n.get_keys().home().hard().inner().to_string(),
            StatMetric::PositiveRatings => i18n.get_keys().home().positive().inner().to_string(),
            StatMetric::NegativeRatings => i18n.get_keys().home().negative().inner().to_string(),
            StatMetric::TotalRatings => i18n.get_keys().home().total_ratings().inner().to_string(),
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
        StatMetric::PositiveRatings => item.positive_ratings(),
        StatMetric::NegativeRatings => item.negative_ratings(),
        StatMetric::TotalRatings => item.total_ratings(),
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
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            "history-modal".to_string()
        } else {
            val
        }
    };
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

    let title = move || metric.get().title(&i18n);

    view! {
        <Modal
            test_id=Signal::derive(test_id_val)
            is_open=is_open_rw
            title=Signal::derive(title)
        >
            <div class="space-y-4">
                {move || if has_data() {
                    view! {
                        <div
                            class="flex justify-center"
                            data-testid=move || {
                                let val = test_id.get();
                                if val.is_empty() { None } else { Some(format!("{}-chart", val)) }
                            }
                        >
                            <LineChart
                                test_id=Signal::derive(move || {
                                    let val = test_id.get();
                                    if val.is_empty() { "history-chart".to_string() } else { format!("{}-chart", val) }
                                })
                                data=chart_data
                                width=420
                                height=200
                            />
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <Text
                            test_id=Signal::derive(move || {
                                let val = test_id.get();
                                if val.is_empty() { "history-empty".to_string() } else { format!("{}-empty", val) }
                            })
                            size=TextSize::Default
                            variant=TypographyVariant::Muted
                            class=Signal::derive(|| "text-center py-8".to_string())
                        >
                            {t!(i18n, home.no_data)}
                        </Text>
                    }.into_any()
                }}
            </div>
        </Modal>
    }
}
