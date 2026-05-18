use chrono::{Datelike, TimeZone};
use origa::domain::{Card, CardType, DailyHistoryItem, KnowledgeSet, NativeLanguage};

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
}

#[derive(Clone)]
pub struct ActivityDataPoint {
    pub date_label: String,
    pub learned: f64,
    pub in_progress: f64,
    pub new_count: f64,
    pub difficult: f64,
}

pub fn compute_today_overview(knowledge_set: &KnowledgeSet) -> TodayOverview {
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

            let meaning = card
                .answer(lang)
                .map(|a| a.text().to_string())
                .unwrap_or_default();

            let card_type = match CardType::from(card) {
                CardType::Kanji => "kanji",
                CardType::Vocabulary => "vocabulary",
                CardType::Grammar => "grammar",
                CardType::Phrase => "vocabulary",
            };

            RecentlyStudiedItem {
                card_id: sc.card_id().to_string(),
                card_type: card_type.to_string(),
                japanese,
                meaning,
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
