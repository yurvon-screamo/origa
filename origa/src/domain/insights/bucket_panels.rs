//! Панели бакетов `InsightsLog`: «Время суток» и «Цена просрочки».

use crate::domain::insights::MIN_BUCKET_SAMPLE;
use crate::domain::knowledge::{HOUR_BUCKET_COUNT, InsightsLog};

/// Пара «попытки, верные» одного бакета.
type Bucket = (u32, u32);

/// Время суток: худшее 4-часовое окно при достаточной выборке.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeOfDayInsight {
    /// Индекс окна 0..6 (0 = 00:00–04:00 … 5 = 20:00–24:00).
    pub worst_bucket: usize,
    pub accuracy_pct: u8,
    pub attempts: u32,
}

/// Цена просрочки: разница точности просроченных ревью и вовремя.
#[derive(Debug, Clone, PartialEq)]
pub struct OverdueInsight {
    /// Разница в процентных пунктах; 0 — просрочка не влияет.
    pub accuracy_drop_pp: u8,
    pub ontime_accuracy_pct: u8,
    pub late_accuracy_pct: u8,
}

/// Точность бакета в процентах при достаточной выборке.
fn bucket_accuracy(buckets: &[Bucket], bucket: usize) -> Option<(u8, u32)> {
    let (attempts, correct) = buckets[bucket];
    if attempts < MIN_BUCKET_SAMPLE {
        return None;
    }
    Some(((correct * 100 / attempts) as u8, attempts))
}

/// #749 панель «Время суток»: худшее по точности 4-часовое окно при
/// достаточной выборке в нём.
pub fn compute_time_of_day(insights: &InsightsLog) -> Option<TimeOfDayInsight> {
    let buckets = insights.hour_buckets();

    let mut worst: Option<(usize, u8, u32)> = None;
    for bucket in 0..HOUR_BUCKET_COUNT {
        let Some((accuracy, attempts)) = bucket_accuracy(&buckets, bucket) else {
            continue;
        };
        let is_worse = match worst {
            None => true,
            Some((_, worst_accuracy, _)) => accuracy < worst_accuracy,
        };
        if is_worse {
            worst = Some((bucket, accuracy, attempts));
        }
    }

    let (worst_bucket, accuracy_pct, attempts) = worst?;
    Some(TimeOfDayInsight {
        worst_bucket,
        accuracy_pct,
        attempts,
    })
}

/// #749 панель «Цена просрочки»: разница точности просроченных ревью
/// (группы 1–3 д / 4–7 д / 8+) и вовремя.
///
/// Min-sample гейт наложен на СУММУ late-групп, а не на каждую отдельно
/// (осознанное отступление от черновика спецификации): панель говорит о цене
/// просрочки как явления — агрегат по всем просроченным даёт покрытие раньше,
/// чем погрупповые гейты, редкие дальние группы не выпадают из вердикта.
pub fn compute_overdue(insights: &InsightsLog) -> Option<OverdueInsight> {
    let buckets = insights.delay_buckets();

    let (ontime_accuracy, _) = bucket_accuracy(&buckets, 0)?;

    let late_attempts: u32 = buckets[1..].iter().map(|(attempts, _)| attempts).sum();
    let late_correct: u32 = buckets[1..].iter().map(|(_, correct)| correct).sum();
    if late_attempts < MIN_BUCKET_SAMPLE {
        return None;
    }
    let late_accuracy_pct = (late_correct * 100 / late_attempts) as u8;

    // Просрочка может и не снижать точность — фиксируем модуль разницы,
    // знак теряется: фраза панели говорит о «цене», а не о пользе.
    let drop = (ontime_accuracy as i16 - late_accuracy_pct as i16).unsigned_abs() as u8;

    Some(OverdueInsight {
        accuracy_drop_pp: drop,
        ontime_accuracy_pct: ontime_accuracy,
        late_accuracy_pct,
    })
}
