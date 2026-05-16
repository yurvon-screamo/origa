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
    pub learning_count: usize,
    pub review_count: usize,
}

impl TodayOverview {
    pub fn total(&self) -> usize {
        self.new_count + self.learning_count + self.review_count
    }
}

#[derive(Clone)]
pub struct RecentlyStudiedItem {
    pub card_id: String,
    pub japanese: String,
    pub reading: String,
    pub meaning: String,
}

#[derive(Clone)]
pub struct ActivityDataPoint {
    pub date_label: String,
    pub learned: f64,
    pub in_progress: f64,
    pub new_count: f64,
}

pub fn compute_today_overview(knowledge_set: &KnowledgeSet) -> TodayOverview {
    let mut overview = TodayOverview::default();

    for study_card in knowledge_set.study_cards().values() {
        let card = study_card.card();
        // Skip phrase cards — they are not part of core study statistics
        if matches!(CardType::from(card), CardType::Phrase) {
            continue;
        }
        let memory = study_card.memory();
        if memory.is_new() {
            overview.new_count += 1;
        } else if memory.is_due() {
            overview.review_count += 1;
        } else if memory.is_in_progress() || memory.is_high_difficulty() {
            overview.learning_count += 1;
        }
    }

    overview
}

pub fn compute_recent_studied(
    knowledge_set: &KnowledgeSet,
    lang: &NativeLanguage,
    limit: usize,
) -> Vec<RecentlyStudiedItem> {
    let mut cards_with_date: Vec<_> = knowledge_set
        .study_cards()
        .values()
        .filter_map(|sc| {
            let last_review = sc.memory().last_review_date()?;
            Some((sc, last_review))
        })
        .collect();

    cards_with_date.sort_by(|a, b| b.1.cmp(&a.1));

    cards_with_date
        .into_iter()
        .take(limit)
        .map(|(sc, _)| {
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

            let reading = match card {
                Card::Kanji(k) => k.kun_readings().first().cloned().unwrap_or_default(),
                _ => String::new(),
            };

            RecentlyStudiedItem {
                card_id: sc.card_id().to_string(),
                japanese,
                reading,
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
            }
        })
        .collect()
}
