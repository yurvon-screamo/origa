//! Панели дневных окон: «Вердикт недели» и «Темп».

use crate::domain::DailyHistoryItem;
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

/// Неделя вердикта: сегодня + 6 предыдущих локальных дней.
pub(crate) const WEEK_DAYS: i64 = 7;
/// Месяц вердикта: сегодня + 29 предыдущих локальных дней.
pub(crate) const MONTH_DAYS: i64 = 30;
/// Окно темпа: 14 полных дней (сегодня исключён — частичный день искажает
/// среднее за день).
const PACE_DAYS: i64 = 14;

/// Вердикт недели: точность 7 дней против предыдущих 30.
#[derive(Debug, Clone, PartialEq)]
pub struct WeekVerdict {
    pub week_accuracy_pct: u8,
    /// Точность предыдущих 30 дней; None — за месяц ещё нет оценок.
    pub month_accuracy_pct: Option<u8>,
    pub kind: WeekVerdictKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeekVerdictKind {
    /// Неделя не хуже месяца (или месяца нет) — «держишь темп».
    Holding,
    /// Неделя просела против месяца.
    Dipping,
}

/// Темп: среднее оценок за календарный день, 14 дней против предыдущих 14.
#[derive(Debug, Clone, PartialEq)]
pub struct PaceInsight {
    pub per_day: u32,
    pub delta: i32,
    /// Всего оценок за окно 1..=14 дней назад.
    pub recent_total: u32,
    /// Всего оценок за окно 15..=28 дней назад.
    pub previous_total: u32,
}

/// Оценки одного локального дня; `days_ago` — 0 для сегодняшнего дня.
struct DayRatings {
    days_ago: i64,
    positive: usize,
    total: usize,
}

/// Канонизированная по локальным дням история: один айтем на день — последний
/// по timestamp. Сортирует вход по timestamp самостоятельно: инвариант
/// «история отсортирована» держится в `StatsTracker::merge`, но функция не
/// полагается на него — дубли одного дня канонизируются в любом порядке.
fn daily_ratings(
    history: &[DailyHistoryItem],
    offset: FixedOffset,
    today: NaiveDate,
) -> Vec<DayRatings> {
    let mut sorted: Vec<&DailyHistoryItem> = history.iter().collect();
    sorted.sort_by_key(|item| item.timestamp());

    let mut by_day: Vec<(NaiveDate, &DailyHistoryItem)> = Vec::new();
    for item in sorted {
        let date = item.timestamp().with_timezone(&offset).date_naive();
        match by_day.last_mut() {
            Some((last_date, last_item)) if *last_date == date => {
                *last_item = item;
            },
            _ => by_day.push((date, item)),
        }
    }

    by_day
        .into_iter()
        .map(|(date, item)| DayRatings {
            days_ago: (today - date).num_days(),
            positive: item.positive_ratings(),
            total: item.total_ratings(),
        })
        .collect()
}

/// Сумма оценок по дням в диапазоне `[from_days_ago, to_days_ago]`.
fn sum_range(days: &[DayRatings], from_days_ago: i64, to_days_ago: i64) -> (usize, usize) {
    days.iter()
        .filter(|day| day.days_ago >= from_days_ago && day.days_ago <= to_days_ago)
        .fold((0usize, 0usize), |(positive, total), day| {
            (positive + day.positive, total + day.total)
        })
}

fn accuracy_pct(positive: usize, total: usize) -> Option<u8> {
    if total == 0 {
        return None;
    }
    Some((positive * 100 / total) as u8)
}

/// #749 панель «Вердикт недели»: точность за 7 локальных дней (включая
/// сегодня — accuracy это ratio, частичный день не искажает) против
/// предыдущих 30.
pub fn compute_week_verdict(
    history: &[DailyHistoryItem],
    offset: FixedOffset,
    now: DateTime<Utc>,
) -> Option<WeekVerdict> {
    compute_window_verdict(history, offset, now, WEEK_DAYS, MONTH_DAYS)
}

/// #749 панель «Месяц»: та же механика, что у недели, но окна 30 дней
/// (включая сегодня) против предыдущих 30.
pub fn compute_month_verdict(
    history: &[DailyHistoryItem],
    offset: FixedOffset,
    now: DateTime<Utc>,
) -> Option<WeekVerdict> {
    compute_window_verdict(history, offset, now, MONTH_DAYS, MONTH_DAYS)
}

/// Общий вердикт окна: точность последних `recent_days` локальных дней
/// против `baseline_days` перед ними.
fn compute_window_verdict(
    history: &[DailyHistoryItem],
    offset: FixedOffset,
    now: DateTime<Utc>,
    recent_days: i64,
    baseline_days: i64,
) -> Option<WeekVerdict> {
    let today = now.with_timezone(&offset).date_naive();
    let days = daily_ratings(history, offset, today);
    if days.is_empty() {
        return None;
    }

    let (recent_positive, recent_total) = sum_range(&days, 0, recent_days - 1);
    let recent_accuracy_pct = accuracy_pct(recent_positive, recent_total)?;

    let (base_positive, base_total) =
        sum_range(&days, recent_days, recent_days + baseline_days - 1);
    let baseline_accuracy_pct = accuracy_pct(base_positive, base_total);

    let kind = match baseline_accuracy_pct {
        Some(baseline) if recent_accuracy_pct < baseline => WeekVerdictKind::Dipping,
        _ => WeekVerdictKind::Holding,
    };

    Some(WeekVerdict {
        week_accuracy_pct: recent_accuracy_pct,
        month_accuracy_pct: baseline_accuracy_pct,
        kind,
    })
}

/// #749 панель «Темп»: среднее оценок за календарный день за 14 полных дней
/// (дни 1..=14) против предыдущих 14 (дни 15..=28). Сегодня исключён
/// сознательно: частичный день занижает среднее.
pub fn compute_pace(
    history: &[DailyHistoryItem],
    offset: FixedOffset,
    now: DateTime<Utc>,
) -> Option<PaceInsight> {
    let today = now.with_timezone(&offset).date_naive();
    let days = daily_ratings(history, offset, today);

    let recent_total = sum_range(&days, 1, PACE_DAYS).1;
    let previous_total = sum_range(&days, PACE_DAYS + 1, PACE_DAYS * 2).1;

    if recent_total == 0 && previous_total == 0 {
        return None;
    }

    let per_day = recent_total as u32 / PACE_DAYS as u32;
    let previous_per_day = previous_total as u32 / PACE_DAYS as u32;

    Some(PaceInsight {
        per_day,
        delta: per_day as i32 - previous_per_day as i32,
        recent_total: recent_total as u32,
        previous_total: previous_total as u32,
    })
}
