use super::*;
use crate::domain::InsightsLog;
use crate::domain::knowledge::DailyStatsUpdate;
use crate::domain::value_objects::Question;
use crate::domain::{
    Card, DailyHistoryItem, KnowledgeSet, NativeLanguage, RateMode, Rating, RatingContext,
    VocabularyCard,
};
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

// --- Вердикт недели ---

#[test]
fn week_verdict_holding_when_week_not_worse_than_month() {
    // Arrange: неделя 80% (4 из 5 в день), месяц 60% (3 из 5 в день)
    let mut history = Vec::new();
    for days_ago in 0..7 {
        history.push(history_item(days_ago, 4, 1));
    }
    for days_ago in 7..37 {
        history.push(history_item(days_ago, 3, 2));
    }

    // Act
    let verdict = compute_week_verdict(&history, offset(3), moscow_now()).unwrap();

    // Assert
    assert_eq!(verdict.week_accuracy_pct, 80);
    assert_eq!(verdict.month_accuracy_pct, Some(60));
    assert_eq!(verdict.kind, WeekVerdictKind::Holding);
}

#[test]
fn week_verdict_dipping_when_week_worse_than_month() {
    // Arrange: неделя 40%, месяц 80%
    let mut history = Vec::new();
    for days_ago in 0..7 {
        history.push(history_item(days_ago, 2, 3));
    }
    for days_ago in 7..37 {
        history.push(history_item(days_ago, 4, 1));
    }

    // Act
    let verdict = compute_week_verdict(&history, offset(3), moscow_now()).unwrap();

    // Assert
    assert_eq!(verdict.kind, WeekVerdictKind::Dipping);
}

#[test]
fn week_verdict_holding_when_month_has_no_ratings() {
    // Arrange: оценки только на этой неделе
    let history = vec![history_item(1, 3, 1)];

    // Act
    let verdict = compute_week_verdict(&history, offset(3), moscow_now()).unwrap();

    // Assert: месяца нет — вердикт без сравнения, Holding
    assert_eq!(verdict.month_accuracy_pct, None);
    assert_eq!(verdict.kind, WeekVerdictKind::Holding);
}

#[test]
fn week_verdict_none_without_any_ratings() {
    // Arrange: пустая история
    let history: Vec<DailyHistoryItem> = vec![];

    // Act
    let verdict = compute_week_verdict(&history, offset(3), moscow_now());

    // Assert
    assert_eq!(verdict, None);
}

#[test]
fn week_verdict_today_partial_day_counts_into_week() {
    // Arrange: единственный день — сегодняшний (days_ago = 0)
    let history = vec![history_item(0, 1, 0)];

    // Act
    let verdict = compute_week_verdict(&history, offset(3), moscow_now()).unwrap();

    // Assert
    assert_eq!(verdict.week_accuracy_pct, 100);
}

// --- Темп ---

#[test]
fn pace_averages_over_calendar_days_excluding_today() {
    // Arrange: в окне 1..=14 — 140 оценок (10/день), в 15..=28 — 70 (5/день)
    let mut history = Vec::new();
    for days_ago in 1..=14 {
        history.push(history_item(days_ago, 10, 0));
    }
    for days_ago in 15..=28 {
        history.push(history_item(days_ago, 5, 0));
    }

    // Act
    let pace = compute_pace(&history, offset(3), moscow_now()).unwrap();

    // Assert
    assert_eq!(pace.per_day, 10);
    assert_eq!(pace.delta, 5);
}

#[test]
fn pace_delta_negative_when_pace_dropped() {
    // Arrange: свежее окно беднее предыдущего
    let mut history = Vec::new();
    for days_ago in 1..=14 {
        history.push(history_item(days_ago, 2, 0));
    }
    for days_ago in 15..=28 {
        history.push(history_item(days_ago, 8, 0));
    }

    // Act
    let pace = compute_pace(&history, offset(3), moscow_now()).unwrap();

    // Assert
    assert_eq!(pace.per_day, 2);
    assert_eq!(pace.delta, -6);
}

#[test]
fn pace_counts_days_without_study_as_zero() {
    // Arrange: активность только через день — знаменатель календарный
    let mut history = Vec::new();
    for days_ago in (1..=14).step_by(2) {
        history.push(history_item(days_ago, 10, 0));
    }

    // Act
    let pace = compute_pace(&history, offset(3), moscow_now()).unwrap();

    // Assert: 7 активных дней × 10 = 70 оценок на 14 календарных дней
    assert_eq!(pace.per_day, 5);
}

#[test]
fn pace_none_without_any_ratings_in_both_windows() {
    // Arrange: одна оценка сегодня — оба окна темпа пусты
    let history = vec![history_item(0, 1, 0)];

    // Act
    let pace = compute_pace(&history, offset(3), moscow_now());

    // Assert
    assert_eq!(pace, None);
}

#[test]
fn pace_none_on_empty_history() {
    // Arrange
    let history: Vec<DailyHistoryItem> = vec![];

    // Act
    let pace = compute_pace(&history, offset(3), moscow_now());

    // Assert
    assert_eq!(pace, None);
}

// --- Канонизация дней ---

#[test]
fn duplicate_day_items_count_once() {
    // Arrange: легаси-дубли одного дня — берётся последний по timestamp.
    // Ранний: 1 из 2; поздний: пустой; сегодня: 1 из 1.
    let mut early = history_item(1, 1, 1);
    early.set_timestamp_for_test(
        moscow_now() - chrono::Duration::days(1) - chrono::Duration::hours(3),
    );
    let late = history_item(1, 0, 0);
    let history = vec![early, late, history_item(0, 1, 0)];

    // Act
    let verdict = compute_week_verdict(&history, offset(3), moscow_now()).unwrap();

    // Assert: поздний айтем дня (0 оценок) каноничен; задвоение дало бы 67%
    assert_eq!(verdict.week_accuracy_pct, 100);
}

// --- Время суток ---

fn insights_with_hour(bucket: usize, attempts: u32, correct: u32) -> InsightsLog {
    let stamp = instant("2026-10-10", (bucket * 4 + 2) as u32);
    let mut log = InsightsLog::default();
    for i in 0..attempts {
        let rating = if i < correct {
            Rating::Good
        } else {
            Rating::Again
        };
        log.record(offset(0), stamp, None, rating);
    }
    log
}

#[test]
fn time_of_day_finds_worst_window_with_enough_sample() {
    // Arrange: окно 4 (16–20) — 50%, окно 2 (08–12) — 100%; дизъюнктные
    // бакеты соединяются max-мержем
    let mut log = insights_with_hour(4, 10, 5);
    log.merge(&insights_with_hour(2, 10, 10));

    // Act
    let insight = compute_time_of_day(&log).unwrap();

    // Assert
    assert_eq!(insight.worst_bucket, 4);
    assert_eq!(insight.accuracy_pct, 50);
    assert_eq!(insight.attempts, 10);
}

#[test]
fn time_of_day_none_below_min_sample() {
    // Arrange: 9 попыток в единственном окне — меньше MIN_BUCKET_SAMPLE
    let log = insights_with_hour(2, 9, 5);

    // Act
    let insight = compute_time_of_day(&log);

    // Assert
    assert_eq!(insight, None);
}

#[test]
fn time_of_day_ties_keep_first_bucket() {
    // Arrange: два окна с одинаковой точностью
    let mut log = insights_with_hour(1, 10, 5);
    log.merge(&insights_with_hour(4, 10, 5));

    // Act
    let insight = compute_time_of_day(&log).unwrap();

    // Assert
    assert_eq!(insight.worst_bucket, 1);
}

// --- Цена просрочки ---

fn insights_with_delay(ontime: (u32, u32), late: (u32, u32)) -> InsightsLog {
    let now = instant("2026-10-10", 12);
    let mut log = InsightsLog::default();
    for i in 0..ontime.0 {
        let rating = if i < ontime.1 {
            Rating::Good
        } else {
            Rating::Again
        };
        log.record(
            offset(0),
            now,
            Some(now - chrono::Duration::hours(1)),
            rating,
        );
    }
    for i in 0..late.0 {
        let rating = if i < late.1 {
            Rating::Good
        } else {
            Rating::Again
        };
        log.record(
            offset(0),
            now,
            Some(now - chrono::Duration::days(2)),
            rating,
        );
    }
    log
}

#[test]
fn overdue_measures_accuracy_drop_in_pp() {
    // Arrange: вовремя 90%, просрочка 50%
    let log = insights_with_delay((10, 9), (10, 5));

    // Act
    let insight = compute_overdue(&log).unwrap();

    // Assert
    assert_eq!(insight.ontime_accuracy_pct, 90);
    assert_eq!(insight.late_accuracy_pct, 50);
    assert_eq!(insight.accuracy_drop_pp, 40);
}

#[test]
fn overdue_reports_drop_even_when_late_accuracy_higher() {
    // Arrange: просрочка точнее вовремя — «цена» отражает модуль разницы
    let log = insights_with_delay((10, 5), (10, 9));

    // Act
    let insight = compute_overdue(&log).unwrap();

    // Assert
    assert_eq!(insight.ontime_accuracy_pct, 50);
    assert_eq!(insight.late_accuracy_pct, 90);
    assert_eq!(insight.accuracy_drop_pp, 40);
}

#[test]
fn overdue_none_without_late_sample() {
    // Arrange: просроченных попыток меньше порога
    let log = insights_with_delay((10, 9), (9, 5));

    // Act
    let insight = compute_overdue(&log);

    // Assert
    assert_eq!(insight, None);
}

#[test]
fn overdue_none_without_ontime_sample() {
    // Arrange: нет группы «вовремя»
    let log = insights_with_delay((0, 0), (10, 5));

    // Act
    let insight = compute_overdue(&log);

    // Assert
    assert_eq!(insight, None);
}

// --- Дорогие слова ---

fn vocab_card(word: &str) -> Card {
    // Глосс через new_with_pos: в unit-тестах переводный словарь не
    // загружен, а hard_word_of пропускает карты без ответа (answer(lang)
    // без reverse_side требует словарь).
    let card = VocabularyCard::new_with_pos(
        Question::new(word.to_string()).unwrap(),
        None,
        Some(Question::new(format!("{word}-gloss")).unwrap()),
    );
    Card::Vocabulary(card)
}

fn seed_troubled_card(set: &mut KnowledgeSet, card: Card, again_streak: u8) {
    let study_card = set.create_card(card).unwrap();
    let card_id = *study_card.card_id();
    set.rate_card(
        card_id,
        Rating::Good,
        RateMode::StandardLesson,
        RatingContext::Explicit,
    )
    .unwrap();
    for _ in 0..again_streak {
        set.rate_card(
            card_id,
            Rating::Again,
            RateMode::StandardLesson,
            RatingContext::Explicit,
        )
        .unwrap();
    }
}

#[test]
fn hardest_words_counts_troubled_share_and_top() {
    // Arrange: 4 изученных слова; у «鳥» 3 провала, у «魚» — 2, остальные здоровы
    let mut set = KnowledgeSet::new();
    seed_troubled_card(&mut set, vocab_card("猫"), 0);
    seed_troubled_card(&mut set, vocab_card("犬"), 0);
    seed_troubled_card(&mut set, vocab_card("鳥"), 3);
    seed_troubled_card(&mut set, vocab_card("魚"), 2);

    // Act
    let insight = compute_hardest_words(&set, &NativeLanguage::English).unwrap();

    // Assert: топ сортирован по убыванию провалов
    assert_eq!(insight.studied_total, 4);
    assert_eq!(insight.troubled_count, 2);
    assert_eq!(insight.one_in, 2);
    assert_eq!(insight.top.len(), 2);
    assert_eq!(insight.top[0].japanese, "鳥");
    assert_eq!(insight.top[0].lapses, 3);
    assert_eq!(insight.top[1].japanese, "魚");
}

#[test]
fn hardest_words_none_when_all_words_healthy() {
    // Arrange
    let mut set = KnowledgeSet::new();
    seed_troubled_card(&mut set, vocab_card("猫"), 0);

    // Act
    let insight = compute_hardest_words(&set, &NativeLanguage::English);

    // Assert
    assert_eq!(insight, None);
}

#[test]
fn hardest_words_none_without_studied_words() {
    // Arrange: только новая карта
    let mut set = KnowledgeSet::new();
    set.create_card(vocab_card("猫")).unwrap();

    // Act
    let insight = compute_hardest_words(&set, &NativeLanguage::English);

    // Assert
    assert_eq!(insight, None);
}

#[test]
fn hardest_words_new_cards_not_counted_in_total() {
    // Arrange: изученное проблемное + новая; новая не в знаменателе
    let mut set = KnowledgeSet::new();
    seed_troubled_card(&mut set, vocab_card("猫"), 2);
    set.create_card(vocab_card("犬")).unwrap();

    // Act
    let insight = compute_hardest_words(&set, &NativeLanguage::English).unwrap();

    // Assert
    assert_eq!(insight.studied_total, 1);
}
