//! Вычислитель панелей-инсайтов (#749): чистые функции от KnowledgeSet.
//!
//! Каждая панель отвечает на один вопрос юзера. Результат — типизированные
//! факты; формулировки фраз живут в UI (i18n). `None` = данных недостаточно,
//! панель показывает нейтральный empty-state.
//!
//! Функции без ambient-зоны: локальная зона передаётся явно (`FixedOffset`),
//! что делает границы окон детерминированными в тестах.
//!
//! Популяция всех панелей — словесные карты (не-Phrase, не-Example), та же
//! конвенция, что у `ComputedStats` и `InsightsLog`.

use crate::domain::knowledge::HOUR_BUCKET_COUNT;
use crate::domain::{Card, CardType, DailyHistoryItem, InsightsLog, KnowledgeSet, NativeLanguage};

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

#[cfg(test)]
#[path = "insights_tests.rs"]
mod tests;

/// Неделя вердикта: сегодня + 6 предыдущих локальных дней.
const WEEK_DAYS: i64 = 7;
/// Окно сравнения вердикта: 30 дней перед неделей.
const MONTH_DAYS: i64 = 30;
/// Окно темпа: 14 полных дней (сегодня исключён — частичный день искажает
/// среднее за день).
const PACE_DAYS: i64 = 14;
/// Минимум попыток в бакете, чтобы точность бакета считалась показательной.
pub const MIN_BUCKET_SAMPLE: u32 = 10;
/// Порог «проблемного» слова: два и более провалов за жизнь. Именно
/// накопительный `lapses`, а не `consecutive_again`: последний сбрасывается
/// при спавне добивания и стрик длиннее одного не видит.
const TROUBLED_LAPSES: u32 = 2;
/// Размер топа дорогих слов.
pub const HARDEST_TOP_SIZE: usize = 3;

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
}

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

/// Одно слово из топа дорогих.
#[derive(Debug, Clone, PartialEq)]
pub struct HardWord {
    pub japanese: String,
    pub meaning: String,
    pub lapses: u32,
}

/// Дорогие слова: доля проблемных слов и топ самых провальных.
#[derive(Debug, Clone, PartialEq)]
pub struct HardestWordsInsight {
    /// Знаменатель «1 из N слов».
    pub one_in: u32,
    pub troubled_count: usize,
    pub studied_total: usize,
    pub top: Vec<HardWord>,
}

/// Оценки одного локального дня; `days_ago` — 0 для сегодняшнего дня.
struct DayRatings {
    days_ago: i64,
    positive: usize,
    total: usize,
}

/// Канонизированная по локальным дням история: один айтем на день — последний
/// по timestamp (конвенция читателей дневных лимитов; легаси-дубли одного дня
/// не должны задваивать счётчики).
fn daily_ratings(
    history: &[DailyHistoryItem],
    offset: FixedOffset,
    today: NaiveDate,
) -> Vec<DayRatings> {
    let mut by_day: Vec<(NaiveDate, &DailyHistoryItem)> = Vec::new();
    for item in history {
        let date = item.timestamp().with_timezone(&offset).date_naive();
        match by_day.last_mut() {
            Some((last_date, last_item)) if *last_date == date => {
                if item.timestamp() >= last_item.timestamp() {
                    *last_item = item;
                }
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
    let today = now.with_timezone(&offset).date_naive();
    let days = daily_ratings(history, offset, today);
    if days.is_empty() {
        return None;
    }

    let (week_positive, week_total) = sum_range(&days, 0, WEEK_DAYS - 1);
    let week_accuracy_pct = accuracy_pct(week_positive, week_total)?;

    let (month_positive, month_total) = sum_range(&days, WEEK_DAYS, WEEK_DAYS + MONTH_DAYS - 1);
    let month_accuracy_pct = accuracy_pct(month_positive, month_total);

    let kind = match month_accuracy_pct {
        Some(month_accuracy) if week_accuracy_pct < month_accuracy => WeekVerdictKind::Dipping,
        _ => WeekVerdictKind::Holding,
    };

    Some(WeekVerdict {
        week_accuracy_pct,
        month_accuracy_pct,
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
    })
}

/// Точность бакета в процентах при достаточной выборке.
fn bucket_accuracy(buckets: &[(u32, u32)], bucket: usize) -> Option<(u8, u32)> {
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

struct WordTrouble {
    lapses: u32,
    difficulty: f64,
}

fn word_trouble(study_card: &crate::domain::StudyCard) -> Option<WordTrouble> {
    let memory = study_card.memory();
    if memory.is_new() {
        return None;
    }
    Some(WordTrouble {
        lapses: memory.lapses(),
        difficulty: memory.difficulty().map(|d| d.value()).unwrap_or(0.0),
    })
}

/// #749 панель «Дорогие слова»: доля проблемных слов (два и более провалов
/// за жизнь среди изученных слов) и топ самых провальных.
pub fn compute_hardest_words(
    knowledge_set: &KnowledgeSet,
    lang: &NativeLanguage,
) -> Option<HardestWordsInsight> {
    let mut studied_total = 0usize;
    let mut troubled: Vec<(&crate::domain::StudyCard, WordTrouble)> = Vec::new();

    for study_card in knowledge_set.study_cards().values() {
        if matches!(
            CardType::from(study_card.card()),
            CardType::Phrase | CardType::Example
        ) {
            continue;
        }
        let Some(trouble) = word_trouble(study_card) else {
            continue;
        };
        studied_total += 1;
        if trouble.lapses >= TROUBLED_LAPSES {
            troubled.push((study_card, trouble));
        }
    }

    if studied_total == 0 || troubled.is_empty() {
        return None;
    }

    troubled.sort_by(|a, b| {
        b.1.lapses
            .cmp(&a.1.lapses)
            .then(b.1.difficulty.total_cmp(&a.1.difficulty))
    });

    // Карта без перевода пропускается в топе (панель рендерит сколько есть);
    // в подсчёте troubled/total она участвует всегда.
    let top: Vec<HardWord> = troubled
        .iter()
        .take(HARDEST_TOP_SIZE)
        .filter_map(|(study_card, trouble)| hard_word_of(study_card, lang, trouble))
        .collect();

    let troubled_count = troubled.len();
    let one_in = (studied_total as f64 / troubled_count as f64).ceil() as u32;

    Some(HardestWordsInsight {
        one_in,
        troubled_count,
        studied_total,
        top,
    })
}

/// Тексты карты; карта без перевода даёт None (не попадает в топ).
fn hard_word_of(
    study_card: &crate::domain::StudyCard,
    lang: &NativeLanguage,
    trouble: &WordTrouble,
) -> Option<HardWord> {
    let card = study_card.card();
    let japanese = match card {
        Card::Vocabulary(v) => v.word().text().to_string(),
        Card::Kanji(k) => k.kanji().text().to_string(),
        Card::Grammar(g) => g.rule_id().to_string(),
        Card::Counter(c) => c.suffix().to_string(),
        Card::Phrase(_) | Card::Example(_) => return None,
    };
    let meaning = card.answer(lang).ok()?.text_projection();

    Some(HardWord {
        japanese,
        meaning,
        lapses: trouble.lapses,
    })
}
