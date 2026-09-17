use super::*;

/// Удаляет из собранного урока незнакомые карты (Exclude): companions могут
/// притаскивать новые слова; фразы освобождены. core_count ужимается по
/// числу выброшенных карт в core-регионе.
pub(crate) fn drop_new_cards(
    data: LessonData,
    knowledge_set: &KnowledgeSet,
    policy: NewCardPolicy,
) -> LessonData {
    let core_count = data.core_count;
    let mut cards = Vec::with_capacity(data.cards.len());
    let mut dropped_in_core = 0usize;
    for (index, (id, lesson_card)) in data.cards.into_iter().enumerate() {
        // Единый источник фактов о карте — StudyCard из knowledge_set.
        let excluded = knowledge_set
            .get_card(id)
            .map(|study_card| {
                excluded_by_new_card_policy(
                    CardType::from(study_card.card()),
                    study_card.memory().is_new(),
                    policy,
                )
            })
            .unwrap_or(false);
        if excluded {
            if index < core_count {
                dropped_in_core += 1;
            }
            continue;
        }
        cards.push((id, lesson_card));
    }
    LessonData {
        cards,
        core_count: core_count.saturating_sub(dropped_in_core),
    }
}
