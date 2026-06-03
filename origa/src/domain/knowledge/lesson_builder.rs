use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};

use ulid::Ulid;

use super::lesson::{LessonCard, LessonData, LessonViewGenerator};
use super::{Card, CardType, KnowledgeSet, StudyCard};
use crate::domain::{JapaneseLevel, JlptContent};

const MIN_LESSON_SIZE: usize = 15;
pub(crate) const MAX_LESSON_SIZE: usize = 50;
const PHRASE_NEW_RATIO: usize = 2;

/// Приоритет карточек без определённого JLPT уровня — ниже всех известных уровней (N1=1)
const UNKNOWN_JLPT_PRIORITY: u8 = 0;

const PHRASE_MAX_PER_LESSON: usize = 5;

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

    let all_lesson_cards: Vec<_> = selected_cards
        .iter()
        .chain(favorite_cards.iter())
        .chain(padding_cards.iter())
        .copied()
        .collect();
    let (lesson_words, lesson_grammar_ids) = extract_lesson_content(&all_lesson_cards);

    let phrase_cards = collect_phrase_cards(
        &all_cards,
        &all_selected_ids,
        knowledge_set,
        daily_new_limit,
        &lesson_words,
        &lesson_grammar_ids,
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

fn extract_lesson_content<'a>(
    cards: impl IntoIterator<Item = &'a (&'a Ulid, &'a StudyCard)>,
) -> (HashSet<String>, HashSet<Ulid>) {
    let mut words = HashSet::new();
    let mut grammar_ids = HashSet::new();

    for (_, study_card) in cards {
        match study_card.card() {
            Card::Vocabulary(vocab) => {
                words.insert(vocab.word().text().to_string());
            },
            Card::Grammar(grammar_card) => {
                grammar_ids.insert(*grammar_card.rule_id());
            },
            Card::Kanji(_) | Card::Phrase(_) => {},
        }
    }

    (words, grammar_ids)
}

fn phrase_lesson_overlap(
    phrase_id: &Ulid,
    lesson_words: &HashSet<String>,
    lesson_grammar_ids: &HashSet<Ulid>,
) -> usize {
    let Some(entry) = crate::dictionary::phrase::get_index_entry(phrase_id) else {
        return 0;
    };

    let vocab_overlap = entry
        .tokens()
        .iter()
        .filter(|t| lesson_words.contains(*t))
        .count();

    let grammar_overlap = entry
        .grammar_rules()
        .iter()
        .filter(|g| lesson_grammar_ids.contains(g))
        .count();

    vocab_overlap + grammar_overlap
}

fn phrase_overlap_from_card(
    study_card: &StudyCard,
    lesson_words: &HashSet<String>,
    lesson_grammar_ids: &HashSet<Ulid>,
) -> usize {
    let Card::Phrase(phrase_card) = study_card.card() else {
        return 0;
    };
    phrase_lesson_overlap(phrase_card.phrase_id(), lesson_words, lesson_grammar_ids)
}

fn sort_phrases_by_overlap<'a>(
    phrases: &mut [(&'a Ulid, &'a StudyCard)],
    lesson_words: &HashSet<String>,
    lesson_grammar_ids: &HashSet<Ulid>,
) {
    phrases.sort_by(|a, b| {
        let overlap_a = phrase_overlap_from_card(a.1, lesson_words, lesson_grammar_ids);
        let overlap_b = phrase_overlap_from_card(b.1, lesson_words, lesson_grammar_ids);
        overlap_b.cmp(&overlap_a).then_with(|| {
            a.1.memory()
                .next_review_date()
                .cmp(&b.1.memory().next_review_date())
        })
    });
}

fn collect_phrase_cards<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    all_selected_ids: &HashSet<Ulid>,
    knowledge_set: &KnowledgeSet,
    daily_new_limit: usize,
    lesson_words: &HashSet<String>,
    lesson_grammar_ids: &HashSet<Ulid>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    let phrase_new_limit = daily_new_limit * PHRASE_NEW_RATIO;
    let phrase_cards_studied = knowledge_set.phrase_cards_studied_today();
    let phrase_new_remaining = phrase_new_limit.saturating_sub(phrase_cards_studied);

    let mut due_phrases: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            !all_selected_ids.contains(id)
                && matches!(card.card(), Card::Phrase(_))
                && card.memory().is_due()
                && !card.memory().is_new()
        })
        .copied()
        .collect();
    sort_phrases_by_overlap(&mut due_phrases, lesson_words, lesson_grammar_ids);

    let mut new_phrases: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            !all_selected_ids.contains(id)
                && matches!(card.card(), Card::Phrase(_))
                && card.memory().is_new()
        })
        .copied()
        .collect();
    sort_phrases_by_overlap(&mut new_phrases, lesson_words, lesson_grammar_ids);
    new_phrases.truncate(phrase_new_remaining);

    let mut phrase_cards = due_phrases;
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
        .into_values()
        .flat_map(|cards| weighted_interleave_by_type(cards))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::knowledge::{GrammarRuleCard, KanjiCard, PhraseCard, VocabularyCard};
    use crate::domain::value_objects::Question;

    fn vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn grammar_card(rule_id: Ulid) -> Card {
        Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id))
    }

    fn phrase_card(phrase_id: Ulid) -> Card {
        Card::Phrase(PhraseCard::new_test_with_id(phrase_id))
    }

    fn kanji_card(kanji: &str) -> Card {
        Card::Kanji(KanjiCard::new_test(kanji.to_string()))
    }

    fn make_study_card(card: Card) -> (Ulid, StudyCard) {
        (Ulid::new(), StudyCard::new(card))
    }

    #[test]
    fn extract_lesson_content_extracts_vocab_and_grammar() {
        let word = "食べる";
        let rule_id = Ulid::new();
        let cards = [
            make_study_card(vocab_card(word)),
            make_study_card(grammar_card(rule_id)),
            make_study_card(phrase_card(Ulid::new())),
            make_study_card(kanji_card("食")),
        ];
        let refs: Vec<(&Ulid, &StudyCard)> = cards.iter().map(|(id, sc)| (id, sc)).collect();

        let (words, grammar_ids) = extract_lesson_content(&refs);

        assert_eq!(words.len(), 1);
        assert!(words.contains(word));
        assert_eq!(grammar_ids.len(), 1);
        assert!(grammar_ids.contains(&rule_id));
    }

    #[test]
    fn extract_lesson_content_returns_empty_when_no_vocab_or_grammar() {
        let cards = [
            make_study_card(phrase_card(Ulid::new())),
            make_study_card(kanji_card("日")),
        ];
        let refs: Vec<(&Ulid, &StudyCard)> = cards.iter().map(|(id, sc)| (id, sc)).collect();

        let (words, grammar_ids) = extract_lesson_content(&refs);

        assert!(words.is_empty());
        assert!(grammar_ids.is_empty());
    }

    #[test]
    fn extract_lesson_content_empty_input() {
        let refs: Vec<(&Ulid, &StudyCard)> = vec![];
        let (words, grammar_ids) = extract_lesson_content(&refs);
        assert!(words.is_empty());
        assert!(grammar_ids.is_empty());
    }

    #[test]
    fn extract_lesson_content_deduplicates_words() {
        let cards = [
            make_study_card(vocab_card("食べる")),
            make_study_card(vocab_card("食べる")),
        ];
        let refs: Vec<(&Ulid, &StudyCard)> = cards.iter().map(|(id, sc)| (id, sc)).collect();

        let (words, _) = extract_lesson_content(&refs);

        assert_eq!(words.len(), 1);
    }

    #[test]
    fn phrase_lesson_overlap_returns_zero_when_index_not_loaded() {
        let phrase_id = Ulid::new();
        let words: HashSet<String> = ["食べる".to_string()].into_iter().collect();

        let overlap = phrase_lesson_overlap(&phrase_id, &words, &HashSet::new());
        assert_eq!(overlap, 0);
    }

    #[test]
    fn phrase_overlap_from_card_returns_zero_for_non_phrase() {
        let (_, sc) = make_study_card(vocab_card("食べる"));
        let words: HashSet<String> = ["食べる".to_string()].into_iter().collect();

        let overlap = phrase_overlap_from_card(&sc, &words, &HashSet::new());
        assert_eq!(overlap, 0);
    }

    #[test]
    fn phrase_overlap_from_card_returns_zero_for_phrase_when_index_not_loaded() {
        let (_, sc) = make_study_card(phrase_card(Ulid::new()));
        let words: HashSet<String> = ["食べる".to_string()].into_iter().collect();

        let overlap = phrase_overlap_from_card(&sc, &words, &HashSet::new());
        assert_eq!(overlap, 0);
    }

    #[test]
    fn sort_phrases_by_overlap_preserves_length_when_equal_overlap() {
        let (id1, sc1) = make_study_card(phrase_card(Ulid::new()));
        let (id2, sc2) = make_study_card(phrase_card(Ulid::new()));

        let mut phrases = vec![(&id1, &sc1), (&id2, &sc2)];
        sort_phrases_by_overlap(&mut phrases, &HashSet::new(), &HashSet::new());

        assert_eq!(phrases.len(), 2);
    }

    #[test]
    fn sort_phrases_by_overlap_does_not_panic_on_empty() {
        let mut phrases: Vec<(&Ulid, &StudyCard)> = vec![];
        sort_phrases_by_overlap(&mut phrases, &HashSet::new(), &HashSet::new());
        assert!(phrases.is_empty());
    }
}
