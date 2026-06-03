use chrono::{Datelike, TimeZone};
use origa::domain::{Card, CardAnswer, CardType, DailyHistoryItem, KnowledgeSet, NativeLanguage};

const RU_MONTHS_SHORT: [&str; 12] = [
    "янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
];
const EN_MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[derive(Clone, Default)]
pub struct TodayOverview {
    pub new_count: usize,
    pub learned_count: usize,
    pub in_progress_count: usize,
    pub difficult_count: usize,
    pub new_delta: Option<i32>,
    pub learned_delta: Option<i32>,
    pub in_progress_delta: Option<i32>,
    pub difficult_delta: Option<i32>,
}

impl TodayOverview {
    pub fn total(&self) -> usize {
        self.new_count + self.learned_count + self.in_progress_count + self.difficult_count
    }
}

#[derive(Clone)]
pub struct RecentlyStudiedItem {
    pub card_id: String,
    pub card_type: String,
    pub japanese: String,
    pub meaning: String,
    pub reading: Option<String>,
    pub short_description: Option<String>,
}

#[derive(Clone)]
pub struct ActivityDataPoint {
    pub date_label: String,
    pub learned: f64,
    pub in_progress: f64,
    pub new_count: f64,
    pub difficult: f64,
}

pub fn compute_today_overview(
    knowledge_set: &KnowledgeSet,
    history: &[DailyHistoryItem],
) -> TodayOverview {
    let mut overview = TodayOverview::default();

    for study_card in knowledge_set.study_cards().values() {
        let card = study_card.card();
        if matches!(CardType::from(card), CardType::Phrase) {
            continue;
        }
        let memory = study_card.memory();
        if memory.is_new() {
            overview.new_count += 1;
        } else if memory.is_high_difficulty() {
            overview.difficult_count += 1;
        } else if memory.is_known_card() {
            overview.learned_count += 1;
        } else {
            overview.in_progress_count += 1;
        }
    }

    if history.len() >= 2 {
        let mut sorted: Vec<_> = history.to_vec();
        sorted.sort_by_key(|a| a.timestamp());
        let today = sorted.last().expect("last item exists");
        let yesterday = sorted
            .iter()
            .rev()
            .nth(1)
            .expect("second-to-last item exists");

        overview.new_delta = Some((today.new_words() as i64 - yesterday.new_words() as i64) as i32);
        overview.learned_delta =
            Some((today.known_words() as i64 - yesterday.known_words() as i64) as i32);
        overview.in_progress_delta =
            Some((today.in_progress_words() as i64 - yesterday.in_progress_words() as i64) as i32);
        overview.difficult_delta = Some(
            (today.high_difficulty_words() as i64 - yesterday.high_difficulty_words() as i64)
                as i32,
        );
    }

    overview
}

pub fn compute_studied_today(
    knowledge_set: &KnowledgeSet,
    lang: &NativeLanguage,
) -> Vec<RecentlyStudiedItem> {
    let today = chrono::Local::now().date_naive();

    let mut cards_today: Vec<_> = knowledge_set
        .study_cards()
        .values()
        .filter(|sc| !matches!(CardType::from(sc.card()), CardType::Phrase))
        .filter(|sc| {
            sc.memory().last_review_date().is_some_and(|last_review| {
                let local_date = chrono::Local
                    .from_utc_datetime(&last_review.naive_utc())
                    .date_naive();
                local_date >= today
            })
        })
        .collect();

    cards_today.sort_by(|a, b| {
        b.memory()
            .last_review_date()
            .cmp(&a.memory().last_review_date())
    });

    cards_today
        .into_iter()
        .map(|sc| {
            let card = sc.card();

            let japanese = card
                .question(lang)
                .map(|q| q.text().to_string())
                .unwrap_or_else(|_| match card {
                    Card::Vocabulary(v) => v.word().text().to_string(),
                    Card::Kanji(k) => k.kanji().text().to_string(),
                    Card::Grammar(g) => g.rule_id().to_string(),
                    Card::Phrase(p) => p.phrase_id().to_string(),
                });

            let meaning = match card.answer(lang) {
                Ok(CardAnswer::Vocabulary { translations, .. }) => translations.join(", "),
                Ok(CardAnswer::Text(s)) => s,
                Err(_) => String::new(),
            };

            let card_type = match CardType::from(card) {
                CardType::Kanji => "kanji",
                CardType::Vocabulary => "vocabulary",
                CardType::Grammar => "grammar",
                CardType::Phrase => "vocabulary",
            };

            let (reading, short_description) = match card {
                Card::Kanji(k) => {
                    let on = k.on_readings();
                    let kun = k.kun_readings();
                    let reading = if on.is_empty() && kun.is_empty() {
                        None
                    } else {
                        let parts: Vec<String> = on.iter().chain(kun.iter()).cloned().collect();
                        Some(parts.join("、"))
                    };
                    (reading, None)
                },
                Card::Grammar(g) => {
                    let short_desc = match g.short_description(lang).ok() {
                        Some(CardAnswer::Vocabulary { translations, .. }) => {
                            Some(translations.join(", "))
                        },
                        Some(CardAnswer::Text(s)) => Some(s),
                        None => None,
                    };
                    (None, short_desc)
                },
                _ => (None, None),
            };

            RecentlyStudiedItem {
                card_id: sc.card_id().to_string(),
                card_type: card_type.to_string(),
                japanese,
                meaning,
                reading,
                short_description,
            }
        })
        .collect()
}

pub fn compute_30day_chart_data(
    history: &[DailyHistoryItem],
    lang: &NativeLanguage,
) -> Vec<ActivityDataPoint> {
    let mut items: Vec<_> = history.to_vec();
    items.sort_by_key(|a| a.timestamp());

    let start = items.len().saturating_sub(30);
    items
        .into_iter()
        .skip(start)
        .map(|item| {
            let local = chrono::Local.from_utc_datetime(&item.timestamp().naive_utc());
            let day = local.day();
            let month_idx = (local.month0() as usize).min(11);
            let month_str = match lang {
                NativeLanguage::Russian => RU_MONTHS_SHORT[month_idx],
                _ => EN_MONTHS_SHORT[month_idx],
            };
            ActivityDataPoint {
                date_label: format!("{} {}", day, month_str),
                learned: item.known_words() as f64,
                in_progress: item.in_progress_words() as f64,
                new_count: item.new_words() as f64,
                difficult: item.high_difficulty_words() as f64,
            }
        })
        .collect()
}

#[derive(Clone, Default)]
pub struct RatingRatio {
    pub percentage: u8,
    pub positive_count: usize,
    pub negative_count: usize,
}

pub fn compute_rating_ratio(history: &[DailyHistoryItem]) -> Option<RatingRatio> {
    let mut positive = 0usize;
    let mut negative = 0usize;

    for item in history {
        positive += item.positive_ratings();
        negative += item.negative_ratings();
    }

    let total = positive + negative;
    if total == 0 {
        return None;
    }

    let percentage = (positive as f64 / total as f64 * 100.0).round() as u8;

    Some(RatingRatio {
        percentage,
        positive_count: positive,
        negative_count: negative,
    })
}

#[derive(Clone, Default)]
pub struct CompletionForecast {
    pub days_remaining: Option<usize>,
    pub is_all_studied: bool,
    pub target_date_label: String,
}

pub fn compute_completion_forecast(
    knowledge_set: &KnowledgeSet,
    history: &[DailyHistoryItem],
    lang: &NativeLanguage,
) -> CompletionForecast {
    use origa::domain::estimate_completion_date;

    let mut new_cards_remaining = 0usize;
    let mut total_cards = 0usize;

    for study_card in knowledge_set.study_cards().values() {
        let card = study_card.card();
        if matches!(CardType::from(card), CardType::Phrase) {
            continue;
        }
        total_cards += 1;
        if study_card.memory().is_new() {
            new_cards_remaining += 1;
        }
    }

    let is_all_studied = total_cards > 0 && new_cards_remaining == 0;

    let (days_remaining, target_date_label) = if new_cards_remaining > 0 {
        match estimate_completion_date(history, new_cards_remaining) {
            Some(date) => {
                let local = chrono::Local.from_utc_datetime(&date.naive_utc());
                let day = local.day();
                let month_idx = (local.month0() as usize).min(11);
                let month_str = match lang {
                    NativeLanguage::Russian => RU_MONTHS_SHORT[month_idx],
                    _ => EN_MONTHS_SHORT[month_idx],
                };
                let date_label = format!("~{} {}", day, month_str);

                let now = chrono::Utc::now();
                let duration = date.signed_duration_since(now);
                let days = (duration.num_hours() as f64 / 24.0).ceil().max(1.0) as usize;

                (Some(days), date_label)
            },
            None => (None, String::new()),
        }
    } else {
        (None, String::new())
    };

    CompletionForecast {
        days_remaining,
        is_all_studied,
        target_date_label,
    }
}
