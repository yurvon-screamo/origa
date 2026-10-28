use super::*;
use crate::domain::knowledge::{DailyHistoryItem, DailyStatsUpdate};

use crate::domain::Rating;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};

fn offset(hours: i32) -> FixedOffset {
    FixedOffset::east_opt(hours * 3600).unwrap()
}

fn instant(date: &str, hour: u32) -> DateTime<Utc> {
    let year = date[0..4].parse::<i32>().unwrap();
    let month = date[5..7].parse::<u32>().unwrap();
    let day = date[8..10].parse::<u32>().unwrap();
    Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap()
}

/// «Сейчас» тестов: 2026-10-13 15:00 по Москве (12:00 UTC).
fn moscow_now() -> DateTime<Utc> {
    instant("2026-10-13", 12)
}

fn minimal_stats_update() -> DailyStatsUpdate {
    DailyStatsUpdate {
        avg_stability: 0.0,
        avg_difficulty: 0.0,
        total_words: 1,
        known_words: 0,
        new_words: 0,
        in_progress_words: 1,
        high_difficulty_words: 0,
        positive_ratings: 0,
        negative_ratings: 0,
        total_ratings: 0,
        new_cards_studied_today: 0,
        phrase_cards_studied_today: 0,
    }
}

fn history_item(days_ago: i64, positive: usize, negative: usize) -> DailyHistoryItem {
    let mut item = DailyHistoryItem::new();
    item.set_timestamp_for_test(moscow_now() - chrono::Duration::days(days_ago));
    for _ in 0..positive {
        item.update(minimal_stats_update(), Rating::Good);
    }
    for _ in 0..negative {
        item.update(minimal_stats_update(), Rating::Again);
    }
    item
}

fn accuracy_at(series: &AccuracySeries, days_ago: u32) -> Option<u8> {
    series
        .days
        .iter()
        .find(|day| day.days_ago == days_ago)
        .and_then(|day| day.accuracy_pct)
}

// --- Структура ряда ---

#[test]
fn series_has_exactly_chart_days_oldest_first() {
    let series = compute_daily_accuracy(&[], offset(3), moscow_now());
    assert_eq!(series.days.len(), CHART_DAYS);
    assert_eq!(series.days.first().unwrap().days_ago, 29);
    assert_eq!(series.days.last().unwrap().days_ago, 0);
}

#[test]
fn empty_history_is_all_none_without_data() {
    let series = compute_daily_accuracy(&[], offset(3), moscow_now());
    assert!(!series.has_data());
    assert!(series.days.iter().all(|day| day.accuracy_pct.is_none()));
}

// --- Точность дня ---

#[test]
fn day_accuracy_is_positive_share_of_that_day() {
    // Arrange: сегодня 3 из 4 (75%), 5 дней назад 1 из 2 (50%)
    let history = vec![history_item(0, 3, 1), history_item(5, 1, 1)];

    // Act
    let series = compute_daily_accuracy(&history, offset(3), moscow_now());

    // Assert
    assert_eq!(accuracy_at(&series, 0), Some(75));
    assert_eq!(accuracy_at(&series, 5), Some(50));
    assert!(series.has_data());
}

#[test]
fn days_without_reviews_are_gaps() {
    // Arrange: оценки только сегодня и 29 дней назад; дни между — разрывы
    let history = vec![history_item(0, 2, 0), history_item(29, 1, 1)];

    // Act
    let series = compute_daily_accuracy(&history, offset(3), moscow_now());

    // Assert
    assert_eq!(accuracy_at(&series, 0), Some(100));
    assert_eq!(accuracy_at(&series, 29), Some(50));
    assert_eq!(accuracy_at(&series, 1), None);
    assert_eq!(accuracy_at(&series, 28), None);
}

#[test]
fn ratings_outside_the_window_are_ignored() {
    // Arrange: день ровно на границе окна и день за ним
    let history = vec![history_item(29, 2, 2), history_item(30, 0, 10)];

    // Act
    let series = compute_daily_accuracy(&history, offset(3), moscow_now());

    // Assert: 30-й день не входит в окно, граничный — входит
    assert_eq!(accuracy_at(&series, 29), Some(50));
    assert!(series.days.iter().all(|day| day.days_ago <= 29));
}

// --- Локальная зона ---

#[test]
fn local_offset_shifts_the_day_boundary() {
    // Arrange: ревью в 2026-10-12 21:00 UTC. При зоне UTC+3 это 13-е
    // (сегодня, days_ago = 0), при зоне UTC-8 — всё ещё 12-е (days_ago = 1).
    let review = instant("2026-10-12", 21);
    let mut item = DailyHistoryItem::new();
    item.set_timestamp_for_test(review);
    item.update(minimal_stats_update(), Rating::Good);
    let history = vec![item];

    // Act
    let east = compute_daily_accuracy(&history, offset(3), instant("2026-10-13", 12));
    let west = compute_daily_accuracy(&history, offset(-8), instant("2026-10-13", 12));

    // Assert
    assert_eq!(accuracy_at(&east, 0), Some(100));
    assert_eq!(accuracy_at(&west, 0), None);
    assert_eq!(accuracy_at(&west, 1), Some(100));
}

// --- Канонизация дублей ---

#[test]
fn legacy_duplicates_canonicalize_to_the_last_item_of_the_day() {
    // Arrange: легаси-дубли одного дня. Победитель — последний по
    // timestamp (у писателя счётчики дня копятся в одном айтеме, последний
    // несёт самые полные значения), а не слияние счётчиков.
    let day_start = moscow_now() - chrono::Duration::days(3);
    let mut partial = DailyHistoryItem::new();
    partial.set_timestamp_for_test(day_start - chrono::Duration::hours(1));
    partial.update(minimal_stats_update(), Rating::Again);

    let mut full = DailyHistoryItem::new();
    full.set_timestamp_for_test(day_start + chrono::Duration::hours(1));
    for _ in 0..4 {
        full.update(minimal_stats_update(), Rating::Good);
    }
    // Вход в «неправильном» порядке: полный раньше неполного по времени.
    let history = vec![full, partial];

    // Act
    let series = compute_daily_accuracy(&history, offset(3), moscow_now());

    // Assert: победил последний по timestamp — 4/4, а не сумма 4/5
    assert_eq!(accuracy_at(&series, 3), Some(100));
}
