use crate::i18n::{Locale, t, use_i18n};
use crate::ui_components::{LineChart, Modal, Text, TextSize, TypographyVariant};
use chrono::{TimeZone, Utc};
use leptos::prelude::*;
use leptos_i18n::I18nContext;
use origa::domain::{MemoryHistory, Rating};
use ulid::Ulid;

fn rating_color(rating: Rating) -> &'static str {
    match rating {
        Rating::Easy => "text-[var(--success)]",
        Rating::Good => "text-[var(--accent-olive)]",
        Rating::Hard => "text-[var(--warning)]",
        Rating::Again => "text-[var(--error)]",
    }
}

fn rating_label(i18n: &I18nContext<Locale>, rating: Rating) -> String {
    match rating {
        Rating::Easy => i18n.get_keys().shared().rating_easy().inner().to_string(),
        Rating::Good => i18n.get_keys().shared().rating_good().inner().to_string(),
        Rating::Hard => i18n.get_keys().shared().rating_hard().inner().to_string(),
        Rating::Again => i18n.get_keys().shared().rating_again().inner().to_string(),
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

    fn label(&self, i18n: &I18nContext<Locale>) -> String {
        rating_label(i18n, self.rating)
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
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(into)] is_open: Signal<bool>,
    memory: MemoryHistory,
    on_close: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
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

    let chart_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{}-chart", val)
        }
    });

    let _on_close_click = Callback::new(move |_: leptos::ev::MouseEvent| {
        on_close.run(());
    });

    view! {
        <Modal
            test_id=test_id
            is_open=is_open_rw
            title=Signal::derive(move || i18n.get_keys().ui().card_history().inner().to_string())
        >
            <div class="card-history-modal">
                {move || if has_data {
                    view! {
                        <div class="flex justify-center" data-testid=chart_test_id>
                            <LineChart
                                data=chart_data
                                width=520
                                height=200
                            />
                        </div>
                        <div class="overflow-y-auto space-y-2">
                            <For
                                each=move || reviews_signal.get()
                                key=|review| review.id
                                children=move |review| {
                                    let color_class = review.color_class();
                                    let label = review.label(&i18n);

                                    view! {
                                        <div class="card-history-item">
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
                            {t!(i18n, ui.card_not_studied)}
                        </Text>
                    }.into_any()
                }}
            </div>
        </Modal>
    }
}
