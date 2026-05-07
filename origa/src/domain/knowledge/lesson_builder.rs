use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};

use ulid::Ulid;

use super::lesson::{LessonCard, LessonData, LessonViewGenerator};
use super::{Card, CardType, KnowledgeSet, StudyCard};
use crate::domain::{JapaneseLevel, JlptContent};

const MIN_LESSON_SIZE: usize = 15;
const MAX_LESSON_SIZE: usize = 50;
const PHRASE_NEW_RATIO: usize = 3;

/// Приоритет карточек без определённого JLPT уровня — ниже всех известных уровней (N1=1)
const UNKNOWN_JLPT_PRIORITY: u8 = 0;

/// Веса типов карточек для interleaving: Vocab:Kanji:Grammar:Phrase ≈ 50:17:17:17.
/// При добавлении нового варианта в CardType — обновить эту константу.
const CARD_TYPE_WEIGHTS: [(CardType, usize); 4] = [
    (CardType::Vocabulary, 3),
    (CardType::Kanji, 1),
    (CardType::Grammar, 1),
    (CardType::Phrase, 1),
];

pub(crate) fn build_lesson(
    knowledge_set: &KnowledgeSet,
    daily_new_limit: usize,
    jlpt_content: &JlptContent,
) -> LessonData {
    let mut all_cards = knowledge_set.study_cards().iter().collect::<Vec<_>>();
    all_cards.sort_by_key(|(_, card)| card.memory().next_review_date());

    let favorite_cards: Vec<_> = all_cards
        .iter()
        .filter(|(_, card)| card.is_favorite())
        .collect();

    let high_diff_limit = MAX_LESSON_SIZE.saturating_sub(favorite_cards.len());
    let mut selected_cards: Vec<_> = all_cards
        .iter()
        .filter(|(_, card)| card.memory().is_due() && card.memory().is_high_difficulty())
        .take(high_diff_limit)
        .copied()
        .collect();

    let new_cards_studied_today = knowledge_set.new_cards_studied_today();
    let remaining_new = daily_new_limit.saturating_sub(new_cards_studied_today);

    let new_cards: Vec<_> = all_cards
        .iter()
        .filter(|(_, card)| card.memory().is_new())
        .copied()
        .collect();

    if !new_cards.is_empty() {
        let distributed = distribute_new_cards(new_cards, jlpt_content);
        let available = MAX_LESSON_SIZE.saturating_sub(selected_cards.len() + favorite_cards.len());
        let daily_remaining = if remaining_new > selected_cards.len() {
            remaining_new.saturating_sub(selected_cards.len())
        } else {
            0
        };
        let allowed_limited = daily_remaining.min(available);
        let mut limited_taken = 0;

        let mut phrase_taken = 0;
        let phrase_new_limit = daily_new_limit / PHRASE_NEW_RATIO;
        for card in distributed {
            let card_type = CardType::from(card.1.card());
            match card_type {
                CardType::Phrase => {
                    if phrase_taken < phrase_new_limit {
                        selected_cards.push(card);
                        phrase_taken += 1;
                    }
                },
                CardType::Vocabulary | CardType::Kanji | CardType::Grammar => {
                    if limited_taken < allowed_limited {
                        selected_cards.push(card);
                        limited_taken += 1;
                    }
                },
            }
        }
    }

    let current_count = selected_cards.len() + favorite_cards.len();
    if current_count < MAX_LESSON_SIZE {
        let remaining = MAX_LESSON_SIZE.saturating_sub(current_count);
        let known_cards: Vec<_> = all_cards
            .iter()
            .filter(|(_, card)| {
                card.memory().is_due()
                    && (card.memory().is_in_progress() || card.memory().is_known_card())
            })
            .take(remaining)
            .copied()
            .collect();
        selected_cards.extend(known_cards);
    }

    let selected_ids: HashSet<_> = selected_cards.iter().map(|(id, _)| **id).collect();
    let favorite_ids: HashSet<_> = favorite_cards.iter().map(|(id, _)| **id).collect();
    let all_selected_ids: HashSet<_> = selected_ids.union(&favorite_ids).copied().collect();

    let total_normal = all_selected_ids.len();
    let mut padding_cards = Vec::new();
    if total_normal < MIN_LESSON_SIZE {
        let needed = MIN_LESSON_SIZE.saturating_sub(total_normal);
        let mut candidates: Vec<_> = all_cards
            .iter()
            .filter(|(id, card)| {
                !all_selected_ids.contains(id) && card.memory().is_high_difficulty()
            })
            .collect();
        candidates.sort_by_key(|(_, card)| card.memory().next_review_date());
        padding_cards = candidates.into_iter().take(needed).collect();
    }

    let padding_ids: HashSet<_> = padding_cards.iter().map(|(id, _)| **id).collect();

    let generator = LessonViewGenerator::new(knowledge_set);

    let mut result: Vec<(Ulid, LessonCard)> = favorite_cards
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            let is_short_term = padding_ids.contains(card_id);
            (**card_id, LessonCard::new(view, is_short_term))
        })
        .collect();

    let selected_lessons: Vec<_> = selected_cards
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            let is_short_term = padding_ids.contains(card_id);
            (**card_id, LessonCard::new(view, is_short_term))
        })
        .collect();

    let padding_lessons: Vec<_> = padding_cards
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            (**card_id, LessonCard::new(view, true))
        })
        .collect();

    result.extend(selected_lessons);
    result.extend(padding_lessons);
    LessonData::reorder_core_first_phrases_last(result)
}

fn resolve_jlpt_level(card: &Card, jlpt_content: &JlptContent) -> Option<JapaneseLevel> {
    jlpt_content.find_level(&card.content_key(), CardType::from(card))
}

fn weighted_interleave_by_type<'a>(
    cards: Vec<(&'a Ulid, &'a StudyCard)>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    use std::collections::VecDeque;

    let mut queues: HashMap<CardType, VecDeque<(&Ulid, &StudyCard)>> = HashMap::new();
    for card in cards {
        let card_type = CardType::from(card.1.card());
        queues.entry(card_type).or_default().push_back(card);
    }

    let pattern: Vec<CardType> = CARD_TYPE_WEIGHTS
        .iter()
        .flat_map(|(ct, w)| std::iter::repeat(*ct).take(*w))
        .collect();

    let total_cards: usize = queues.values().map(|q| q.len()).sum();
    let mut result = Vec::with_capacity(total_cards);
    let mut pattern_idx = 0;
    let mut empty_rounds = 0;

    while result.len() < total_cards {
        let card_type = pattern[pattern_idx % pattern.len()];
        pattern_idx += 1;

        if let Some(queue) = queues.get_mut(&card_type) {
            if let Some(card) = queue.pop_front() {
                result.push(card);
                empty_rounds = 0;
                continue;
            }
        }

        empty_rounds += 1;
        // Если за полный цикл паттерна не извлекли ни одной карточки —
        // значит остались типы, не представленные в CARD_TYPE_WEIGHTS
        if empty_rounds >= pattern.len() {
            queues
                .values_mut()
                .flat_map(|q| q.drain(..))
                .for_each(|card| result.push(card));
            break;
        }
    }

    result
}

fn distribute_new_cards<'a>(
    new_cards: Vec<(&'a Ulid, &'a StudyCard)>,
    jlpt_content: &JlptContent,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    // Reverse: N5(5) → наивысший приоритет → первый ключ в BTreeMap
    let mut groups: BTreeMap<Reverse<u8>, Vec<(&Ulid, &StudyCard)>> = BTreeMap::new();
    for card in new_cards {
        let priority = resolve_jlpt_level(card.1.card(), jlpt_content)
            .map(|l| l.as_number())
            .unwrap_or(UNKNOWN_JLPT_PRIORITY);
        groups.entry(Reverse(priority)).or_default().push(card);
    }

    groups
        .into_iter()
        .flat_map(|(_, cards)| weighted_interleave_by_type(cards))
        .collect()
}
