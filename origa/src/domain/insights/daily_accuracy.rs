//! Ежедневная точность для графика на главной: ряд за 30 локальных дней.

use crate::domain::DailyHistoryItem;
use chrono::{DateTime, FixedOffset, Utc};

/// Окно графика: сегодня + 29 предыдущих локальных дней.
pub const CHART_DAYS: usize = 30;

/// Точность одного локального дня.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayAccuracy {
    /// 0 — сегодняшний день, 29 — месяц назад.
    pub days_ago: u32,
    /// Доля позитивных оценок дня в процентах; `None` — оценок не было.
    pub accuracy_pct: Option<u8>,
    /// Сколько оценок стояло за днём (для подсказок у точки).
    pub total_ratings: u32,
}

/// Ряд точности по дням от старых к новым; всегда ровно [`CHART_DAYS`]
/// айтемов — дни без оценок несут `None`, график рисует разрывы.
#[derive(Debug, Clone, PartialEq)]
pub struct AccuracySeries {
    pub days: Vec<DayAccuracy>,
}

impl AccuracySeries {
    /// Есть ли в окне хоть одна оценка.
    pub fn has_data(&self) -> bool {
        self.days.iter().any(|day| day.accuracy_pct.is_some())
    }
}

/// Точность по дням за [`CHART_DAYS`] локальных дней.
///
/// Рейтинг дня — доля позитивных оценок канонического айтема дня (история
/// копит оценки дня в одном айтеме; легаси-дубли канонизируются в последний
/// по timestamp — та же конвенция, что у читающих путей дневных лимитов).
pub fn compute_daily_accuracy(
    history: &[DailyHistoryItem],
    offset: FixedOffset,
    now: DateTime<Utc>,
) -> AccuracySeries {
    let today = now.with_timezone(&offset).date_naive();

    // Канонизация по локальным дням: один айтем на день — последний по
    // timestamp. Сортировка самостоятельная: инвариант «история
    // отсортирована» держится в StatsTracker, функция на него не опирается.
    let mut sorted: Vec<&DailyHistoryItem> = history.iter().collect();
    sorted.sort_by_key(|item| item.timestamp());

    let mut by_day: Vec<(i64, &DailyHistoryItem)> = Vec::new();
    for item in sorted {
        let date = item.timestamp().with_timezone(&offset).date_naive();
        let days_ago = (today - date).num_days();
        match by_day.last_mut() {
            Some((last_ago, last_item)) if *last_ago == days_ago => *last_item = item,
            _ => by_day.push((days_ago, item)),
        }
    }

    let mut days = Vec::with_capacity(CHART_DAYS);
    for days_ago in (0..CHART_DAYS as i64).rev() {
        let (positive, total) = by_day
            .iter()
            .find(|(ago, _)| *ago == days_ago)
            .map(|(_, item)| (item.positive_ratings(), item.total_ratings()))
            .unwrap_or((0, 0));
        days.push(DayAccuracy {
            days_ago: days_ago as u32,
            accuracy_pct: positive
                .checked_mul(100)
                .and_then(|scaled| scaled.checked_div(total))
                .map(|pct| pct as u8),
            total_ratings: total as u32,
        });
    }
    AccuracySeries { days }
}
