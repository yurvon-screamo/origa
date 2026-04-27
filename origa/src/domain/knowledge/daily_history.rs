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
    lessons_completed: usize,

    positive_ratings: usize,
    negative_ratings: usize,
    total_ratings: usize,
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

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        avg_stability: f64,
        avg_difficulty: f64,
        total_words: usize,
        known_words: usize,
        new_words: usize,
        in_progress_words: usize,
        high_difficulty_words: usize,
        rating: Rating,
        new_cards_studied_today: u32,
    ) {
        self.update_stats(
            avg_stability,
            avg_difficulty,
            total_words,
            known_words,
            new_words,
            in_progress_words,
            high_difficulty_words,
            self.positive_ratings
                + match rating {
                    Rating::Easy | Rating::Good => 1,
                    Rating::Hard | Rating::Again => 0,
                },
            self.negative_ratings
                + match rating {
                    Rating::Hard | Rating::Again => 1,
                    Rating::Easy | Rating::Good => 0,
                },
            self.total_ratings + 1,
            new_cards_studied_today,
        );
        self.lessons_completed += 1;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_stats(
        &mut self,
        avg_stability: f64,
        avg_difficulty: f64,
        total_words: usize,
        known_words: usize,
        new_words: usize,
        in_progress_words: usize,
        high_difficulty_words: usize,
        positive_ratings: usize,
        negative_ratings: usize,
        total_ratings: usize,
        new_cards_studied_today: u32,
    ) {
        self.avg_stability = Some(avg_stability);
        self.avg_difficulty = Some(avg_difficulty);
        self.total_words = total_words;
        self.known_words = known_words;
        self.new_words = new_words;
        self.in_progress_words = in_progress_words;
        self.high_difficulty_words = high_difficulty_words;
        self.positive_ratings = positive_ratings;
        self.negative_ratings = negative_ratings;
        self.total_ratings = total_ratings;
        self.new_cards_studied_today = new_cards_studied_today;
    }

    pub fn merge_with(&mut self, other: &DailyHistoryItem) {
        self.lessons_completed = self.lessons_completed.max(other.lessons_completed);
        self.positive_ratings = self.positive_ratings.max(other.positive_ratings);
        self.negative_ratings = self.negative_ratings.max(other.negative_ratings);
        self.total_ratings = self.total_ratings.max(other.total_ratings);
        self.new_cards_studied_today = self
            .new_cards_studied_today
            .max(other.new_cards_studied_today);

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
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_item(
        timestamp: DateTime<Utc>,
        lessons: usize,
        known: usize,
        total: usize,
    ) -> DailyHistoryItem {
        let mut item = DailyHistoryItem::new();
        item.update_stats(
            0.5, // avg_stability
            0.3, // avg_difficulty
            total,
            known,
            total - known, // new_words
            0,             // in_progress
            0,             // high_difficulty
            0,             // positive_ratings
            0,             // negative_ratings
            0,             // total_ratings
            0,             // new_cards_studied_today
        );
        for _ in 0..lessons {
            item.lessons_completed += 1;
        }
        item.timestamp = timestamp;
        item
    }

    #[test]
    fn test_daily_history_item_new() {
        let item = DailyHistoryItem::new();

        assert_eq!(item.lessons_completed(), 0);
        assert_eq!(item.known_words(), 0);
        assert_eq!(item.total_words(), 0);
        assert_eq!(item.avg_stability(), None);
        assert_eq!(item.avg_difficulty(), None);
    }

    #[test]
    fn test_daily_history_item_getters() {
        let now = Utc::now();
        let item = create_test_item(now, 1, 3, 8);

        assert_eq!(item.avg_stability(), Some(0.5));
        assert_eq!(item.avg_difficulty(), Some(0.3));
        assert_eq!(item.new_words(), 5); // 8 - 3
        assert_eq!(item.in_progress_words(), 0);
        assert_eq!(item.high_difficulty_words(), 0);
    }

    #[test]
    fn test_daily_history_item_update() {
        let mut item = DailyHistoryItem::new();

        item.update(0.5, 0.3, 5, 10, 2, 3, 1, Rating::Good, 0);

        assert_eq!(item.lessons_completed(), 1);
        assert_eq!(item.total_words(), 5);
        assert_eq!(item.known_words(), 10);
        assert_eq!(item.new_words(), 2);
        assert_eq!(item.in_progress_words(), 3);
        assert_eq!(item.high_difficulty_words(), 1);
        assert_eq!(item.avg_stability(), Some(0.5));
        assert_eq!(item.avg_difficulty(), Some(0.3));
        assert_eq!(item.positive_ratings(), 1);
        assert_eq!(item.negative_ratings(), 0);
        assert_eq!(item.total_ratings(), 1);
    }

    #[test]
    fn test_merge_with_takes_higher_lessons() {
        let now = Utc::now();
        let mut item1 = create_test_item(now, 2, 5, 10);
        let item2 = create_test_item(now, 5, 3, 8);

        item1.merge_with(&item2);

        assert_eq!(item1.lessons_completed(), 5);
    }

    #[test]
    fn test_merge_with_preserves_known_words_when_other_older() {
        let now = Utc::now();
        let older = now - Duration::seconds(100);
        let mut item1 = create_test_item(now, 2, 5, 10);
        let item2 = create_test_item(older, 5, 8, 12);

        let known_before = item1.known_words();
        item1.merge_with(&item2);

        assert_eq!(item1.known_words(), known_before);
    }

    #[test]
    fn test_merge_with_updates_timestamp_when_newer() {
        let now = Utc::now();
        let newer = now + Duration::seconds(100);
        let mut item1 = create_test_item(now, 2, 5, 10);
        let item2 = create_test_item(newer, 5, 3, 8);

        item1.merge_with(&item2);

        assert_eq!(item1.timestamp(), newer);
        assert_eq!(item1.avg_stability(), Some(0.5));
        assert_eq!(item1.avg_difficulty(), Some(0.3));
    }

    #[test]
    fn test_merge_with_does_not_update_timestamp_when_older() {
        let now = Utc::now();
        let older = now - Duration::seconds(100);
        let mut item1 = create_test_item(now, 2, 5, 10);
        let item2 = create_test_item(older, 5, 3, 8);

        let timestamp_before = item1.timestamp();
        item1.merge_with(&item2);

        assert_eq!(item1.timestamp(), timestamp_before);
    }

    #[test]
    fn test_merge_with_updates_stats_when_newer() {
        let now = Utc::now();
        let newer = now + Duration::seconds(100);
        let mut item1 = create_test_item(now, 2, 5, 10);
        let mut item2 = create_test_item(newer, 5, 3, 8);
        item2.update_stats(0.8, 0.6, 8, 3, 5, 0, 0, 0, 0, 0, 0);

        item1.merge_with(&item2);

        assert_eq!(item1.avg_stability(), Some(0.8));
        assert_eq!(item1.avg_difficulty(), Some(0.6));
    }

    #[test]
    fn test_merge_with_preserves_stats_when_other_older() {
        let now = Utc::now();
        let older = now - Duration::seconds(100);
        let mut item1 = create_test_item(now, 2, 5, 10);
        let mut item2 = create_test_item(older, 5, 3, 8);
        item2.update_stats(0.8, 0.6, 8, 3, 5, 0, 0, 0, 0, 0, 0);

        let stability_before = item1.avg_stability();
        let difficulty_before = item1.avg_difficulty();
        item1.merge_with(&item2);

        assert_eq!(item1.avg_stability(), stability_before);
        assert_eq!(item1.avg_difficulty(), difficulty_before);
    }

    #[test]
    fn test_update_increments_positive_ratings_on_good() {
        let mut item = DailyHistoryItem::new();
        item.update(0.5, 0.3, 5, 10, 2, 3, 1, Rating::Good, 0);

        assert_eq!(item.positive_ratings(), 1);
        assert_eq!(item.negative_ratings(), 0);
        assert_eq!(item.total_ratings(), 1);
    }

    #[test]
    fn test_update_increments_positive_ratings_on_easy() {
        let mut item = DailyHistoryItem::new();
        item.update(0.5, 0.3, 5, 10, 2, 3, 1, Rating::Easy, 0);

        assert_eq!(item.positive_ratings(), 1);
        assert_eq!(item.negative_ratings(), 0);
        assert_eq!(item.total_ratings(), 1);
    }

    #[test]
    fn test_update_increments_negative_ratings_on_again() {
        let mut item = DailyHistoryItem::new();
        item.update(0.5, 0.3, 5, 10, 2, 3, 1, Rating::Again, 0);

        assert_eq!(item.positive_ratings(), 0);
        assert_eq!(item.negative_ratings(), 1);
        assert_eq!(item.total_ratings(), 1);
    }

    #[test]
    fn test_update_increments_negative_ratings_on_hard() {
        let mut item = DailyHistoryItem::new();
        item.update(0.5, 0.3, 5, 10, 2, 3, 1, Rating::Hard, 0);

        assert_eq!(item.positive_ratings(), 0);
        assert_eq!(item.negative_ratings(), 1);
        assert_eq!(item.total_ratings(), 1);
    }

    #[test]
    fn test_update_accumulates_ratings() {
        let mut item = DailyHistoryItem::new();
        item.update(0.5, 0.3, 5, 10, 2, 3, 1, Rating::Good, 0);
        item.update(0.5, 0.3, 5, 10, 2, 3, 1, Rating::Easy, 0);
        item.update(0.5, 0.3, 5, 10, 2, 3, 1, Rating::Again, 0);

        assert_eq!(item.positive_ratings(), 2);
        assert_eq!(item.negative_ratings(), 1);
        assert_eq!(item.total_ratings(), 3);
    }

    #[test]
    fn test_merge_with_takes_max_ratings() {
        let now = Utc::now();
        let mut item1 = DailyHistoryItem::new();
        item1.update_stats(0.5, 0.3, 10, 5, 5, 0, 0, 3, 2, 5, 0);
        item1.timestamp = now;

        let mut item2 = DailyHistoryItem::new();
        item2.update_stats(0.6, 0.4, 12, 6, 6, 0, 0, 5, 3, 8, 0);
        item2.timestamp = now;

        item1.merge_with(&item2);

        assert_eq!(item1.positive_ratings(), 5);
        assert_eq!(item1.negative_ratings(), 3);
        assert_eq!(item1.total_ratings(), 8);
    }

    #[test]
    fn test_new_cards_studied_today_default() {
        let item = DailyHistoryItem::new();
        assert_eq!(item.new_cards_studied_today(), 0);
    }

    #[test]
    fn test_increment_new_cards_studied() {
        let mut item = DailyHistoryItem::new();
        item.increment_new_cards_studied();
        item.increment_new_cards_studied();
        assert_eq!(item.new_cards_studied_today(), 2);
    }

    #[test]
    fn test_merge_with_takes_max_new_cards_studied() {
        let now = Utc::now();
        let mut item1 = create_test_item(now, 2, 5, 10);
        let mut item2 = create_test_item(now, 5, 3, 8);
        item2.increment_new_cards_studied();
        item2.increment_new_cards_studied();
        item2.increment_new_cards_studied();
        item1.merge_with(&item2);
        assert_eq!(item1.new_cards_studied_today(), 3);
    }

    fn create_test_item_with_new_studied(
        timestamp: DateTime<Utc>,
        new_studied: u32,
    ) -> DailyHistoryItem {
        let mut item = DailyHistoryItem::new();
        item.timestamp = timestamp;
        for _ in 0..new_studied {
            item.increment_new_cards_studied();
        }
        item
    }

    #[test]
    fn test_estimate_returns_none_when_no_new_cards() {
        let history = vec![create_test_item_with_new_studied(
            Utc::now() - Duration::days(1),
            5,
        )];
        assert!(estimate_completion_date(&history, 0).is_none());
    }

    #[test]
    fn test_estimate_returns_none_when_no_history() {
        assert!(estimate_completion_date(&[], 100).is_none());
    }

    #[test]
    fn test_estimate_returns_none_when_all_zero_studied() {
        let history = vec![create_test_item_with_new_studied(
            Utc::now() - Duration::days(1),
            0,
        )];
        assert!(estimate_completion_date(&history, 100).is_none());
    }

    #[test]
    fn test_estimate_returns_none_when_excludes_today() {
        let history = vec![create_test_item_with_new_studied(Utc::now(), 10)];
        assert!(estimate_completion_date(&history, 100).is_none());
    }

    #[test]
    fn test_estimate_basic_calculation() {
        let history = vec![create_test_item_with_new_studied(
            Utc::now() - Duration::days(1),
            5,
        )];
        let result = estimate_completion_date(&history, 50);
        assert!(result.is_some());
        let expected = Utc::now() + Duration::days(12);
        assert_eq!(result.unwrap().date_naive(), expected.date_naive());
    }

    #[test]
    fn test_estimate_averages_over_multiple_days() {
        let history = vec![
            create_test_item_with_new_studied(Utc::now() - Duration::days(2), 10),
            create_test_item_with_new_studied(Utc::now() - Duration::days(1), 20),
        ];
        let result = estimate_completion_date(&history, 30);
        assert!(result.is_some());
        let expected = Utc::now() + Duration::days(3);
        assert_eq!(result.unwrap().date_naive(), expected.date_naive());
    }

    #[test]
    fn test_estimate_skips_zero_study_days() {
        let history = vec![
            create_test_item_with_new_studied(Utc::now() - Duration::days(3), 10),
            create_test_item_with_new_studied(Utc::now() - Duration::days(2), 0),
            create_test_item_with_new_studied(Utc::now() - Duration::days(1), 10),
        ];
        let result = estimate_completion_date(&history, 20);
        assert!(result.is_some());
        let expected = Utc::now() + Duration::days(3);
        assert_eq!(result.unwrap().date_naive(), expected.date_naive());
    }

    #[test]
    fn test_estimate_uses_last_10_non_today_records() {
        let history: Vec<DailyHistoryItem> = (1..=15)
            .map(|i| create_test_item_with_new_studied(Utc::now() - Duration::days(i), 5))
            .collect();
        let result = estimate_completion_date(&history, 50);
        assert!(result.is_some());
        let expected = Utc::now() + Duration::days(12);
        assert_eq!(result.unwrap().date_naive(), expected.date_naive());
    }
}
