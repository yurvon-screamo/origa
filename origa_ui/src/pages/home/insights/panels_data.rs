use origa::domain::insights::{
    HardestWordsInsight, OverdueInsight, compute_hardest_words, compute_overdue,
};
use origa::domain::insights::{
    PaceInsight, TimeOfDayInsight, WeekVerdict, compute_month_verdict, compute_pace,
    compute_time_of_day, compute_week_verdict,
};
use origa::domain::{KnowledgeSet, NativeLanguage, local_day};

use chrono::Utc;

/// Снимок всех панелей-инсайтов для отрисовки секции.
#[derive(Clone, Default)]
pub struct InsightsPanels {
    pub week: Option<WeekVerdict>,
    pub month: Option<WeekVerdict>,
    pub pace: Option<PaceInsight>,
    pub time_of_day: Option<TimeOfDayInsight>,
    pub overdue: Option<OverdueInsight>,
    pub hardest_words: Option<HardestWordsInsight>,
}

/// Вычисляет все панели за один проход по данным набора знаний.
pub fn compute_insights_panels(
    knowledge_set: &KnowledgeSet,
    lang: &NativeLanguage,
) -> InsightsPanels {
    let offset = local_day::local_offset();
    let now = Utc::now();

    InsightsPanels {
        week: compute_week_verdict(knowledge_set.lesson_history(), offset, now),
        month: compute_month_verdict(knowledge_set.lesson_history(), offset, now),
        pace: compute_pace(knowledge_set.lesson_history(), offset, now),
        time_of_day: compute_time_of_day(knowledge_set.insights()),
        overdue: compute_overdue(knowledge_set.insights()),
        hardest_words: compute_hardest_words(knowledge_set, lang),
    }
}
