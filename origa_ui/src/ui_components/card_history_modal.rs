use crate::ui_components::{LineChart, Modal, Text, TextSize, TypographyVariant};
use chrono::{TimeZone, Utc};
use leptos::prelude::*;
use origa::domain::{MemoryHistory, Rating};
use ulid::Ulid;

fn rating_color(rating: Rating) -> &'static str {
    match rating {
        Rating::Easy => "text-green-600",
        Rating::Good => "text-blue-600",
        Rating::Hard => "text-orange-600",
        Rating::Again => "text-red-600",
    }
}

fn rating_label(rating: Rating) -> &'static str {
    match rating {
        Rating::Easy => "Легко",
        Rating::Good => "Хорошо",
        Rating::Hard => "Сложно",
        Rating::Again => "Снова",
    }
}

fn format_interval(days: i64) -> String {
    if days == 0 {
        "0d".to_string()
    } else if days < 7 {
        format!("{}d", days)
    } else if days < 30 {
        format!("{}w", days / 7)
    } else if days < 365 {
        format!("{}m", days / 30)
    } else {
        format!("{}y", days / 365)
    }
}

fn format_date(timestamp: chrono::DateTime<Utc>) -> String {
    let local = chrono::Local.from_utc_datetime(&timestamp.naive_utc());
    local.format("%d.%m %H:%M").to_string()
}

#[derive(Clone, Copy)]
struct ReviewItem {
    id: Ulid,
    rating: Rating,
    timestamp: chrono::DateTime<Utc>,
    interval_days: i64,
}

impl ReviewItem {
    fn color_class(&self) -> &'static str {
        rating_color(self.rating)
    }

    fn label(&self) -> &'static str {
        rating_label(self.rating)
    }

    fn date_str(&self) -> String {
        format_date(self.timestamp)
    }

    fn interval_str(&self) -> String {
        format_interval(self.interval_days)
    }
}

#[component]
pub fn CardHistoryModal(
    #[prop(into)] is_open: Signal<bool>,
    memory: MemoryHistory,
    on_close: Callback<()>,
) -> impl IntoView {
    let is_open_rw = RwSignal::new(is_open.get_untracked());

    Effect::new(move || {
        is_open_rw.set(is_open.get());
    });

    let review_items: Vec<ReviewItem> = memory
        .reviews()
        .iter()
        .map(|r| ReviewItem {
            id: r.id(),
            rating: r.rating(),
            timestamp: r.timestamp(),
            interval_days: r.interval().num_days(),
        })
        .collect();

    let recent_reviews: Vec<ReviewItem> = review_items.iter().rev().take(20).cloned().collect();

    let chart_items: Vec<(String, f64)> = review_items
        .iter()
        .map(|r| (r.date_str(), r.interval_days as f64))
        .collect();

    let has_data = !review_items.is_empty();

    let chart_data = Signal::derive(move || chart_items.clone());
    let reviews_signal = Signal::derive(move || recent_reviews.clone());

    let _on_close_click = Callback::new(move |_: leptos::ev::MouseEvent| {
        on_close.run(());
    });

    view! {
        <Modal
            is_open=is_open_rw
            title=Signal::derive(|| "История карточки".to_string())
        >
            <div class="space-y-4">
                {move || if has_data {
                    view! {
                        <div class="flex justify-center">
                            <LineChart
                                data=chart_data
                                width=380
                                height=150
                            />
                        </div>
                        <div class="overflow-y-auto space-y-2">
                            <For
                                each=move || reviews_signal.get()
                                key=|review| review.id
                                children=move |review| {
                                    let color_class = review.color_class();
                                    let label = review.label();

                                    view! {
                                        <div class="flex justify-between items-center py-1 px-2 rounded bg-[var(--bg-secondary)]">
                                            <Text
                                                size=TextSize::Small
                                                variant=TypographyVariant::Primary
                                            >
                                                {review.date_str()}
                                            </Text>
                                            <div class="flex items-center gap-4">
                                                <span class=Signal::derive(move || format!("font-mono text-sm {}", color_class))>
                                                    {label}
                                                </span>
                                                <Text
                                                    size=TextSize::Small
                                                    variant=TypographyVariant::Muted
                                                >
                                                    {review.interval_str()}
                                                </Text>
                                            </div>
                                        </div>
                                    }
                                }
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
                            "Карточка ещё не изучалась"
                        </Text>
                    }.into_any()
                }}
            </div>
        </Modal>
    }
}
