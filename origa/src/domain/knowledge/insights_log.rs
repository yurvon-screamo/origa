use serde::{Deserialize, Serialize};

use crate::domain::memory::Rating;

use chrono::{DateTime, FixedOffset, Timelike, Utc};

/// Шесть 4-часовых окон локальных суток: 00–04, 04–08, 08–12, 12–16, 16–20, 20–24.
pub const HOUR_BUCKET_COUNT: usize = 6;

/// Четыре группы просрочки: вовремя, 1–3 дня, 4–7 дней, 8+ дней.
pub const DELAY_BUCKET_COUNT: usize = 4;

/// Ревью попадает в группу просрочки только начиная с полных суток опоздания:
/// опоздание меньше 24 часов считается «вовремя».
const MIN_LATE_DAYS: i64 = 1;
const DELAY_BUCKET_1_3_MAX_DAYS: i64 = 3;
const DELAY_BUCKET_4_7_MAX_DAYS: i64 = 7;

/// Точка пары «попытки, верные» одного бакета.
pub type Bucket = (u32, u32);

/// Константные агрегаты точности ответов для панелей-инсайтов (#749).
///
/// Размер фиксирован (~100 байт в памяти) и не растёт со временем — урок
/// ADR-034: per-review журнал достигал 95% wire payload knowledge_set.
/// Из этого следует ограничение v1: бакеты кумулятивные за всё время,
/// ретроспектива «последние N дней» по ним невозможна.
///
/// Популяция записей — ревью словесных карт: не-Phrase и не-Example. Это
/// та же конвенция, что у `ComputedStats`/`recalculate_daily_stats`; с
/// live-путём `DailyHistoryItem` (исключает только Phrase) возможен малый
/// расхождение по Example — известная несогласованность самого
/// `DailyHistoryItem`, здесь выбрана каноничная «словесная» конвенция.
///
/// Cross-device merge — поэлементный `max()` по конвенции репо для
/// счётчиков (`MemoryHistory`, `DailyHistoryItem::merge_with`): при
/// offline-дивергенции возможен недосчёт, планирование не затрагивает.
///
/// Онбординг-скоринг и implicit dual-rating грамматики записываются как
/// обычные ревью — та же семантика подсчёта, что у дневных рейтингов.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InsightsLog {
    hour_attempts: [u32; HOUR_BUCKET_COUNT],
    hour_correct: [u32; HOUR_BUCKET_COUNT],
    delay_attempts: [u32; DELAY_BUCKET_COUNT],
    delay_correct: [u32; DELAY_BUCKET_COUNT],
}

/// Индекс 4-часового окна локальных суток, содержащего `now` в зоне `offset`.
pub fn hour_bucket_of(offset: FixedOffset, now: DateTime<Utc>) -> usize {
    let local_hour = now.with_timezone(&offset).hour();
    (local_hour / 4) as usize
}

/// Индекс группы просрочки ревью, пришедшего в `now` при `next_review_date`
/// `due` на момент показа. Просрочка меньше суток — «вовремя».
pub fn delay_bucket_of(now: DateTime<Utc>, due: DateTime<Utc>) -> usize {
    let days_late = (now - due).num_days().max(0);
    if days_late < MIN_LATE_DAYS {
        0
    } else if days_late <= DELAY_BUCKET_1_3_MAX_DAYS {
        1
    } else if days_late <= DELAY_BUCKET_4_7_MAX_DAYS {
        2
    } else {
        3
    }
}

impl InsightsLog {
    /// Записывает одно ревью. Вызывается только для словесных карт
    /// (не-Phrase, не-Example) — см. doc-comment структуры.
    pub fn record(
        &mut self,
        offset: FixedOffset,
        now: DateTime<Utc>,
        pre_review_due: Option<DateTime<Utc>>,
        rating: Rating,
    ) {
        let is_correct = matches!(rating, Rating::Easy | Rating::Good);

        let hour_bucket = hour_bucket_of(offset, now);
        self.hour_attempts[hour_bucket] += 1;
        if is_correct {
            self.hour_correct[hour_bucket] += 1;
        }

        if let Some(due) = pre_review_due {
            let delay_bucket = delay_bucket_of(now, due);
            self.delay_attempts[delay_bucket] += 1;
            if is_correct {
                self.delay_correct[delay_bucket] += 1;
            }
        }
    }

    pub fn hour_buckets(&self) -> [Bucket; HOUR_BUCKET_COUNT] {
        let mut buckets = [(0u32, 0u32); HOUR_BUCKET_COUNT];
        for ((bucket, attempts), correct) in buckets
            .iter_mut()
            .zip(&self.hour_attempts)
            .zip(&self.hour_correct)
        {
            *bucket = (*attempts, *correct);
        }
        buckets
    }

    pub fn delay_buckets(&self) -> [Bucket; DELAY_BUCKET_COUNT] {
        let mut buckets = [(0u32, 0u32); DELAY_BUCKET_COUNT];
        for ((bucket, attempts), correct) in buckets
            .iter_mut()
            .zip(&self.delay_attempts)
            .zip(&self.delay_correct)
        {
            *bucket = (*attempts, *correct);
        }
        buckets
    }

    /// Общее число записанных ревью (сумма попыток по часовым бакетам).
    pub fn total_reviews(&self) -> u32 {
        self.hour_attempts.iter().sum()
    }

    /// Поэлементный max по всем бакетам (конвенция репо для счётчиков).
    pub fn merge(&mut self, other: &InsightsLog) {
        for i in 0..HOUR_BUCKET_COUNT {
            self.hour_attempts[i] = self.hour_attempts[i].max(other.hour_attempts[i]);
            self.hour_correct[i] = self.hour_correct[i].max(other.hour_correct[i]);
        }
        for i in 0..DELAY_BUCKET_COUNT {
            self.delay_attempts[i] = self.delay_attempts[i].max(other.delay_attempts[i]);
            self.delay_correct[i] = self.delay_correct[i].max(other.delay_correct[i]);
        }
    }
}

#[cfg(test)]
#[path = "insights_log_tests.rs"]
mod tests;
