use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::memory::Rating;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyHistoryItem {
    timestamp: DateTime<Utc>,
    avg_stability: Option<f64>,
    avg_difficulty: Option<f64>,

    total_words: usize,
    new_words: usize,
    known_words: usize,
    in_progress_words: usize,
    high_difficulty_words: usize,
    #[serde(default)]
    new_cards_studied_today: u32,
    #[serde(default)]
    phrase_cards_studied_today: u32,
    lessons_completed: usize,

    positive_ratings: usize,
    negative_ratings: usize,
    total_ratings: usize,
}

pub(crate) struct DailyStatsUpdate {
    pub(crate) avg_stability: f64,
    pub(crate) avg_difficulty: f64,
    pub(crate) total_words: usize,
    pub(crate) known_words: usize,
    pub(crate) new_words: usize,
    pub(crate) in_progress_words: usize,
    pub(crate) high_difficulty_words: usize,
    pub(crate) positive_ratings: usize,
    pub(crate) negative_ratings: usize,
    pub(crate) total_ratings: usize,
    pub(crate) new_cards_studied_today: u32,
    pub(crate) phrase_cards_studied_today: u32,
}

impl Default for DailyHistoryItem {
    fn default() -> Self {
        Self::new()
    }
}

impl DailyHistoryItem {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            avg_stability: None,
            avg_difficulty: None,
            total_words: 0,
            new_words: 0,
            known_words: 0,
            in_progress_words: 0,
            high_difficulty_words: 0,
            new_cards_studied_today: 0,
            phrase_cards_studied_today: 0,
            lessons_completed: 0,
            positive_ratings: 0,
            negative_ratings: 0,
            total_ratings: 0,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn avg_stability(&self) -> Option<f64> {
        self.avg_stability
    }

    pub fn avg_difficulty(&self) -> Option<f64> {
        self.avg_difficulty
    }

    pub fn total_words(&self) -> usize {
        self.total_words
    }

    pub fn known_words(&self) -> usize {
        self.known_words
    }

    pub fn new_words(&self) -> usize {
        self.new_words
    }

    pub fn in_progress_words(&self) -> usize {
        self.in_progress_words
    }

    pub fn high_difficulty_words(&self) -> usize {
        self.high_difficulty_words
    }

    pub fn new_cards_studied_today(&self) -> u32 {
        self.new_cards_studied_today
    }

    pub fn increment_new_cards_studied(&mut self) {
        self.new_cards_studied_today += 1;
    }

    pub fn phrase_cards_studied_today(&self) -> u32 {
        self.phrase_cards_studied_today
    }

    pub fn increment_phrase_cards_studied(&mut self) {
        self.phrase_cards_studied_today += 1;
    }

    pub fn lessons_completed(&self) -> usize {
        self.lessons_completed
    }

    pub fn positive_ratings(&self) -> usize {
        self.positive_ratings
    }

    pub fn negative_ratings(&self) -> usize {
        self.negative_ratings
    }

    pub fn total_ratings(&self) -> usize {
        self.total_ratings
    }

    pub(crate) fn update(&mut self, stats: DailyStatsUpdate, rating: Rating) {
        self.update_stats(DailyStatsUpdate {
            positive_ratings: self.positive_ratings
                + match rating {
                    Rating::Easy | Rating::Good => 1,
                    Rating::Hard | Rating::Again => 0,
                },
            negative_ratings: self.negative_ratings
                + match rating {
                    Rating::Hard | Rating::Again => 1,
                    Rating::Easy | Rating::Good => 0,
                },
            total_ratings: self.total_ratings + 1,
            ..stats
        });
        self.lessons_completed += 1;
    }

    pub(crate) fn update_stats(&mut self, stats: DailyStatsUpdate) {
        self.avg_stability = Some(stats.avg_stability);
        self.avg_difficulty = Some(stats.avg_difficulty);
        self.total_words = stats.total_words;
        self.known_words = stats.known_words;
        self.new_words = stats.new_words;
        self.in_progress_words = stats.in_progress_words;
        self.high_difficulty_words = stats.high_difficulty_words;
        self.positive_ratings = stats.positive_ratings;
        self.negative_ratings = stats.negative_ratings;
        self.total_ratings = stats.total_ratings;
        self.new_cards_studied_today = stats.new_cards_studied_today;
        self.phrase_cards_studied_today = stats.phrase_cards_studied_today;
    }

    pub fn merge_with(&mut self, other: &DailyHistoryItem) {
        self.lessons_completed = self.lessons_completed.max(other.lessons_completed);
        self.positive_ratings = self.positive_ratings.max(other.positive_ratings);
        self.negative_ratings = self.negative_ratings.max(other.negative_ratings);
        self.total_ratings = self.total_ratings.max(other.total_ratings);
        self.new_cards_studied_today = self
            .new_cards_studied_today
            .max(other.new_cards_studied_today);
        self.phrase_cards_studied_today = self
            .phrase_cards_studied_today
            .max(other.phrase_cards_studied_today);

        if other.timestamp > self.timestamp {
            self.timestamp = other.timestamp;
            self.avg_stability = other.avg_stability;
            self.avg_difficulty = other.avg_difficulty;
        }
    }
}

/// Оценивает дату завершения изучения всех оставшихся новых карточек
/// на основе средней дневной скорости за последние 10 дней (исключая текущий).
/// Возвращает `None` если нет оставшихся карточек или недостаточно данных.
pub fn estimate_completion_date(
    history: &[DailyHistoryItem],
    new_cards_remaining: usize,
) -> Option<DateTime<Utc>> {
    if new_cards_remaining == 0 {
        return None;
    }

    let today = Utc::now().date_naive();

    let studied: Vec<&DailyHistoryItem> = history
        .iter()
        .rev()
        .filter(|item| item.timestamp().date_naive() != today)
        .take(10)
        .filter(|item| item.new_cards_studied_today() > 0)
        .collect();

    if studied.is_empty() {
        return None;
    }

    let total: u32 = studied
        .iter()
        .map(|item| item.new_cards_studied_today())
        .sum();
    let avg = total as f64 / studied.len() as f64;
    let days = (new_cards_remaining as f64 / avg * 1.15).ceil() as i64;

    Some(Utc::now() + Duration::days(days))
}

#[cfg(test)]
#[path = "daily_history_tests.rs"]
mod tests;
