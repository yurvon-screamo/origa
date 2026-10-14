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

mod bucket_panels;
mod daily_windows;
mod hardest_words;

pub use bucket_panels::{OverdueInsight, TimeOfDayInsight, compute_overdue, compute_time_of_day};
pub use daily_windows::{
    PaceInsight, WeekVerdict, WeekVerdictKind, compute_month_verdict, compute_pace,
    compute_week_verdict,
};
pub use hardest_words::{HardWord, HardestWordsInsight, compute_hardest_words};

/// Минимум попыток в бакете, чтобы точность бакета считалась показательной.
pub const MIN_BUCKET_SAMPLE: u32 = 10;

#[cfg(test)]
#[path = "insights_tests.rs"]
mod tests;
