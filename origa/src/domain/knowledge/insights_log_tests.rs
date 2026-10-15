use super::*;
use chrono::TimeZone;
use rstest::rstest;

fn offset(hours: i32) -> FixedOffset {
    FixedOffset::east_opt(hours * 3600).unwrap()
}

fn instant(date: &str, hour: u32) -> DateTime<Utc> {
    let year = date[0..4].parse::<i32>().unwrap();
    let month = date[5..7].parse::<u32>().unwrap();
    let day = date[8..10].parse::<u32>().unwrap();
    Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap()
}

fn due_before(now: DateTime<Utc>, days: f64) -> DateTime<Utc> {
    now - chrono::Duration::seconds((days * 86_400.0) as i64)
}

#[rstest]
#[case::first_window_midnight("2026-10-10", 0, 0)]
#[case::first_window_3am("2026-10-10", 3, 0)]
#[case::second_window_4am("2026-10-10", 4, 1)]
#[case::third_window_noon("2026-10-10", 12, 3)]
#[case::last_window_23("2026-10-10", 23, 5)]
fn hour_bucket_maps_local_hour_to_window(
    #[case] date: &str,
    #[case] local_hour: u32,
    #[case] expected: usize,
) {
    // Arrange: UTC-инстанты, сдвинутые так, чтобы ЛОКАЛЬНЫЙ час был нужным
    let utc_hour = ((local_hour as i32 - 3).rem_euclid(24)) as u32;
    let now = instant(date, utc_hour);

    // Act
    let bucket = hour_bucket_of(offset(3), now);

    // Assert
    assert_eq!(bucket, expected);
}

#[rstest]
#[case::due_in_future_is_ontime(-1.0, 0)]
#[case::late_23h59_is_ontime(0.999, 0)]
#[case::late_24h_is_1_3(1.0 + 1.0 / 24.0, 1)]
#[case::late_3d_is_1_3(3.0, 1)]
#[case::late_3d23h_still_1_3(3.0 + 23.0 / 24.0, 1)]
#[case::late_4d_is_4_7(4.0, 2)]
#[case::late_7d23h_still_4_7(7.0 + 23.0 / 24.0, 2)]
#[case::late_8d_is_8plus(8.0, 3)]
#[case::late_30d_is_8plus(30.0, 3)]
fn delay_bucket_maps_lateness_to_group(#[case] late_days: f64, #[case] expected: usize) {
    // Arrange: неполные сутки просрочки (floor < 1) — «вовремя»
    let now = instant("2026-10-10", 12);
    let due = due_before(now, late_days);

    // Act
    let bucket = delay_bucket_of(now, due);

    // Assert
    assert_eq!(bucket, expected);
}

#[test]
fn record_correct_rating_counts_attempt_and_correct() {
    // Arrange
    let now = instant("2026-10-10", 9);
    let mut log = InsightsLog::default();

    // Act
    log.record(offset(0), now, None, Rating::Good);

    // Assert
    assert_eq!(log.hour_buckets()[2], (1, 1));
    assert_eq!(log.delay_buckets()[0], (0, 0));
    assert_eq!(log.total_reviews(), 1);
}

#[test]
fn record_wrong_rating_counts_attempt_only() {
    // Arrange
    let now = instant("2026-10-10", 21);
    let mut log = InsightsLog::default();

    // Act
    log.record(offset(0), now, None, Rating::Again);

    // Assert
    assert_eq!(log.hour_buckets()[5], (1, 0));
    assert_eq!(log.total_reviews(), 1);
}

#[test]
fn hard_rating_counts_as_incorrect() {
    // Arrange: Hard — negative в конвенции DailyHistoryItem
    let now = instant("2026-10-10", 12);
    let mut log = InsightsLog::default();

    // Act
    log.record(offset(0), now, None, Rating::Hard);

    // Assert
    assert_eq!(log.hour_buckets()[3], (1, 0));
}

#[test]
fn record_with_due_writes_delay_bucket() {
    // Arrange
    let now = instant("2026-10-10", 12);
    let mut log = InsightsLog::default();

    // Act: просрочка на 2 дня → группа 1 (1–3 дня)
    log.record(offset(0), now, Some(due_before(now, 2.0)), Rating::Easy);

    // Assert
    assert_eq!(log.delay_buckets()[1], (1, 1));
    assert_eq!(log.hour_buckets()[3], (1, 1));
}

#[test]
fn record_without_due_writes_no_delay_bucket() {
    // Arrange: новое слово — due нет
    let now = instant("2026-10-10", 12);
    let mut log = InsightsLog::default();

    // Act
    log.record(offset(0), now, None, Rating::Good);

    // Assert
    assert_eq!(log.delay_buckets(), [(0, 0); DELAY_BUCKET_COUNT]);
}

#[test]
fn merge_takes_elementwise_max() {
    // Arrange: дивергенция — у левого больше попыток, у правого больше верных
    let now = instant("2026-10-10", 12);
    let mut left = InsightsLog::default();
    left.record(offset(0), now, None, Rating::Again);
    left.record(offset(0), now, None, Rating::Again);

    let mut right = InsightsLog::default();
    right.record(offset(0), now, None, Rating::Good);

    // Act
    left.merge(&right);

    // Assert: попытки max(2,1)=2, верные max(0,1)=1
    assert_eq!(left.hour_buckets()[3], (2, 1));
}

#[test]
fn merge_empty_into_non_empty_preserves_counts() {
    // Arrange
    let now = instant("2026-10-10", 12);
    let mut log = InsightsLog::default();
    log.record(offset(0), now, None, Rating::Good);

    // Act
    log.merge(&InsightsLog::default());

    // Assert
    assert_eq!(log.total_reviews(), 1);
}

#[test]
fn legacy_knowledge_set_without_insights_field_deserializes_to_default() {
    // Arrange: формат данных до панелей-инсайтов — поля insights нет
    let json = r#"{"study_cards": {}, "lesson_history": []}"#;

    // Act
    let set: super::super::KnowledgeSet = serde_json::from_str(json).unwrap();

    // Assert
    assert_eq!(set.insights(), &InsightsLog::default());
}

#[test]
fn serialized_roundtrip_preserves_buckets() {
    // Arrange
    let now = instant("2026-10-10", 12);
    let mut log = InsightsLog::default();
    log.record(offset(0), now, Some(due_before(now, 5.0)), Rating::Good);

    // Act
    let json = serde_json::to_string(&log).unwrap();
    let restored: InsightsLog = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(restored, log);
}
