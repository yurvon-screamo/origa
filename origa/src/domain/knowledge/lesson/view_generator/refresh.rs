use rand::Rng;

use super::super::types::{LessonCard, LessonCardView, LessonData};
use super::LessonViewGenerator;
use crate::domain::Card;
use crate::domain::knowledge::KnowledgeSet;
use crate::domain::value_objects::NativeLanguage;

/// Re-selects review phrase views after lazily loaded details arrive.
///
/// The lesson is built before the phrase detail cache is filled, so a
/// listen view can degrade to `Normal` at build time. Callers must invoke
/// this after loading details and before publishing the lesson cards.
/// New phrase cards keep the view chosen at build time.
pub fn refresh_phrase_views(
    lesson: &mut LessonData,
    knowledge_set: &KnowledgeSet,
    native_language: NativeLanguage,
    rng: &mut impl Rng,
) -> usize {
    let mut generator = LessonViewGenerator::new(knowledge_set, native_language);
    let mut changed = 0;

    for (card_id, lesson_card) in &mut lesson.cards {
        if !matches!(lesson_card.card(), Card::Phrase(_)) {
            continue;
        }
        let Some(study_card) = knowledge_set.get_card(*card_id) else {
            continue;
        };
        if study_card.is_new() {
            continue;
        }

        let view = generator.apply_view(study_card, false, rng);
        if !matches!(view, LessonCardView::PhraseListen { .. }) {
            continue;
        }

        *lesson_card = LessonCard::new(*card_id, view, lesson_card.is_short_term());
        changed += 1;
    }

    changed
}
