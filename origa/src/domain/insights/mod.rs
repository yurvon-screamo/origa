//! Вычислитель графика точности (#749 → переработка fl2O037UR7): чистые
//! функции от KnowledgeSet.
//!
//! Результат — типизированные факты; формулировки живут в UI (i18n).
//! Функции без ambient-зоны: локальная зона передаётся явно (`FixedOffset`),
//! что делает границы дней детерминированными в тестах.
//!
//! Популяция — все оценки дня из `lesson_history`; сырой лог ревью
//! (`InsightsLog`) продолжает писаться в домене независимо от UI.

mod daily_accuracy;

pub use daily_accuracy::{AccuracySeries, CHART_DAYS, DayAccuracy, compute_daily_accuracy};

#[cfg(test)]
#[path = "insights_tests.rs"]
mod tests;
