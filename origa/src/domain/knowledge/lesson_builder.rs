use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};

use ulid::Ulid;

use super::lesson::{LessonCard, LessonData, LessonViewGenerator};
use super::{Card, CardType, KnowledgeSet, StudyCard};
use crate::domain::{JapaneseLevel, JlptContent};

const MIN_LESSON_SIZE: usize = 15;
const MAX_LESSON_SIZE: usize = 50;
const PHRASE_NEW_RATIO: usize = 2;

/// Приоритет карточек без определённого JLPT уровня — ниже всех известных уровней (N1=1)
const UNKNOWN_JLPT_PRIORITY: u8 = 0;

const PHRASE_MAX_PER_LESSON: usize = 7;

/// Веса типов карточек для interleaving: Vocab:Kanji:Grammar ≈ 60:20:20.
/// При добавлении нового варианта в CardType — обновить эту константу.
const CARD_TYPE_WEIGHTS: [(CardType, usize); 3] = [
    (CardType::Vocabulary, 3),
    (CardType::Kanji, 1),
    (CardType::Grammar, 1),
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
        .copied()
        .collect();

    let favorite_ids: HashSet<Ulid> = favorite_cards.iter().map(|(id, _)| **id).collect();

    let mut selected_cards = collect_core_high_difficulty(&all_cards, &favorite_ids);
    collect_core_new_cards(
        &all_cards,
        &mut selected_cards,
        &favorite_ids,
        knowledge_set.new_cards_studied_today(),
        daily_new_limit,
        jlpt_content,
    );
    fill_core_due_known(&all_cards, &mut selected_cards, &favorite_ids);

    let all_selected_ids = build_selected_ids(&selected_cards, &favorite_cards);
    let padding_cards = collect_padding(&all_cards, &all_selected_ids);

    let phrase_cards = collect_phrase_cards(
        &all_cards,
        &all_selected_ids,
        knowledge_set,
        daily_new_limit,
    );

    build_lesson_result(
        &favorite_cards,
        &selected_cards,
        &padding_cards,
        &phrase_cards,
        knowledge_set,
    )
}

fn collect_core_high_difficulty<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    favorite_ids: &HashSet<Ulid>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    let limit = MAX_LESSON_SIZE.saturating_sub(favorite_ids.len());
    all_cards
        .iter()
        .filter(|(id, card)| {
            !favorite_ids.contains(id)
                && !matches!(card.card(), Card::Phrase(_))
                && card.memory().is_due()
                && card.memory().is_high_difficulty()
        })
        .take(limit)
        .copied()
        .collect()
}

fn collect_core_new_cards<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    selected_cards: &mut Vec<(&'a Ulid, &'a StudyCard)>,
    favorite_ids: &HashSet<Ulid>,
    new_cards_studied_today: usize,
    daily_new_limit: usize,
    jlpt_content: &JlptContent,
) {
    let new_core_cards: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            !favorite_ids.contains(id)
                && !matches!(card.card(), Card::Phrase(_))
                && card.memory().is_new()
        })
        .copied()
        .collect();

    if new_core_cards.is_empty() {
        return;
    }

    let distributed = distribute_new_cards(new_core_cards, jlpt_content);
    let available = MAX_LESSON_SIZE.saturating_sub(selected_cards.len() + favorite_ids.len());
    let daily_remaining = daily_new_limit
        .saturating_sub(new_cards_studied_today)
        .saturating_sub(
            selected_cards
                .iter()
                .filter(|(_, c)| c.memory().is_new())
                .count(),
        );
    let allowed = daily_remaining.min(available);

    for card in distributed {
        if selected_cards.len() >= allowed {
            break;
        }
        selected_cards.push(card);
    }
}

fn fill_core_due_known<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    selected_cards: &mut Vec<(&'a Ulid, &'a StudyCard)>,
    favorite_ids: &HashSet<Ulid>,
) {
    let current_count = selected_cards.len() + favorite_ids.len();
    let remaining = MAX_LESSON_SIZE.saturating_sub(current_count);
    if remaining == 0 {
        return;
    }

    let due_known: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            !favorite_ids.contains(id)
                && !matches!(card.card(), Card::Phrase(_))
                && card.memory().is_due()
                && (card.memory().is_in_progress() || card.memory().is_known_card())
        })
        .take(remaining)
        .copied()
        .collect();
    selected_cards.extend(due_known);
}

fn collect_padding<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    all_selected_ids: &HashSet<Ulid>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    if all_selected_ids.len() >= MIN_LESSON_SIZE {
        return Vec::new();
    }
    let needed = MIN_LESSON_SIZE.saturating_sub(all_selected_ids.len());
    let mut candidates: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            !all_selected_ids.contains(id)
                && !matches!(card.card(), Card::Phrase(_))
                && card.memory().is_high_difficulty()
        })
        .copied()
        .collect();
    candidates.sort_by_key(|(_, card)| card.memory().next_review_date());
    candidates.into_iter().take(needed).collect()
}

fn collect_phrase_cards<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    all_selected_ids: &HashSet<Ulid>,
    knowledge_set: &KnowledgeSet,
    daily_new_limit: usize,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    let phrase_new_limit = daily_new_limit * PHRASE_NEW_RATIO;
    let phrase_cards_studied = knowledge_set.phrase_cards_studied_today();
    let phrase_new_remaining = phrase_new_limit.saturating_sub(phrase_cards_studied);

    let mut phrase_cards: Vec<_> = Vec::new();

    let due_phrases: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            !all_selected_ids.contains(id)
                && matches!(card.card(), Card::Phrase(_))
                && card.memory().is_due()
                && !card.memory().is_new()
        })
        .copied()
        .collect();
    phrase_cards.extend(due_phrases);

    let new_phrases: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            !all_selected_ids.contains(id)
                && matches!(card.card(), Card::Phrase(_))
                && card.memory().is_new()
        })
        .take(phrase_new_remaining)
        .copied()
        .collect();
    phrase_cards.extend(new_phrases);

    phrase_cards.truncate(PHRASE_MAX_PER_LESSON);
    phrase_cards
}

fn build_selected_ids(
    selected_cards: &[(&Ulid, &StudyCard)],
    favorite_cards: &[(&Ulid, &StudyCard)],
) -> HashSet<Ulid> {
    let selected_ids: HashSet<_> = selected_cards.iter().map(|(id, _)| **id).collect();
    let favorite_ids: HashSet<_> = favorite_cards.iter().map(|(id, _)| **id).collect();
    selected_ids.union(&favorite_ids).copied().collect()
}

fn build_lesson_result(
    favorite_cards: &[(&Ulid, &StudyCard)],
    selected_cards: &[(&Ulid, &StudyCard)],
    padding_cards: &[(&Ulid, &StudyCard)],
    phrase_cards: &[(&Ulid, &StudyCard)],
    knowledge_set: &KnowledgeSet,
) -> LessonData {
    let padding_ids: HashSet<_> = padding_cards.iter().map(|(id, _)| **id).collect();
    let mut generator = LessonViewGenerator::new(knowledge_set);

    let favorite_lessons: Vec<_> = favorite_cards
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

    let phrase_lessons: Vec<_> = phrase_cards
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            (**card_id, LessonCard::new(view, false))
        })
        .collect();

    let mut result = favorite_lessons;
    result.extend(selected_lessons);
    result.extend(padding_lessons);
    result.extend(phrase_lessons);
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
