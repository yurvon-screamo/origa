use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};

use rand::seq::SliceRandom;
use ulid::Ulid;

use super::lesson::{LessonCard, LessonData, LessonViewGenerator};
use super::{Card, CardType, KnowledgeSet, StudyCard};
use crate::domain::{JapaneseLevel, JlptContent};

const MIN_LESSON_SIZE: usize = 15;
pub(crate) const MAX_LESSON_SIZE: usize = 30;
const PHRASE_NEW_RATIO: usize = 2;

/// Приоритет карточек без определённого JLPT уровня — ниже всех известных уровней (N1=1)
const UNKNOWN_JLPT_PRIORITY: u8 = 0;

const PHRASE_MAX_PER_LESSON: usize = 5;

/// Tail phrases only reinforce already-mastered material: no single known word
/// may appear in more than this many tail phrases, otherwise a frequent word
/// (e.g. する) would crowd out the entire tail.
pub(crate) const MAX_PHRASES_PER_WORD_IN_TAIL: usize = 1;

/// How many phrases may be interleaved next to a single anchor word inside the
/// core section. Two cards let the learner meet the word in context shortly
/// after the standalone review without saturating the lesson.
const INTERLEAVED_PHRASES_PER_WORD: usize = 2;

/// Minimum number of other cards that must sit between an anchor word and its
/// first interleaved phrase, so the phrase does not leak the answer back into
/// the word rating. Degrades to "as late as possible" on short lessons.
const INTERLEAVING_GAP: usize = 2;

/// Веса типов карточек для interleaving: Vocab:Kanji:Grammar ≈ 80:10:10.
/// При добавлении нового варианта в CardType — обновить эту константу.
const CARD_TYPE_WEIGHTS: [(CardType, usize); 3] = [
    (CardType::Vocabulary, 8),
    (CardType::Kanji, 1),
    (CardType::Grammar, 1),
];

/// Remaining new-phrase allowance shared between the interleaved and tail
/// sections. Encapsulates `PHRASE_NEW_RATIO` so callers cannot reach past it.
pub(crate) fn compute_phrase_new_budget(daily_new_limit: usize, studied: usize) -> usize {
    (daily_new_limit * PHRASE_NEW_RATIO).saturating_sub(studied)
}

/// Collects the lesson core (favorites, due/new/known cards, padding) WITHOUT
/// any phrases. The core is shuffled here so downstream interleaving sees a
/// stable order. Phrases are attached later by `add_interleaved_phrases` and
/// `add_tail_phrases`.
pub(crate) fn build_lesson_core(
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

    let mut cards = build_core_lesson_cards(
        &favorite_cards,
        &selected_cards,
        &padding_cards,
        knowledge_set,
    );
    cards.shuffle(&mut rand::rng());
    let core_count = cards.len();

    LessonData { cards, core_count }
}

/// Core-section eligibility predicate shared by the high-difficulty, new and
/// due-known collectors: a card is a core candidate when it is neither a
/// user favorite (already pinned) nor a phrase (phrases enter the lesson via
/// the interleaved/tail pipelines, not the core).
fn is_core_candidate(id: &Ulid, card: &StudyCard, favorite_ids: &HashSet<Ulid>) -> bool {
    !favorite_ids.contains(id) && !matches!(card.card(), Card::Phrase(_))
}

fn collect_core_high_difficulty<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    favorite_ids: &HashSet<Ulid>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    let limit = MAX_LESSON_SIZE.saturating_sub(favorite_ids.len());
    all_cards
        .iter()
        .filter(|(id, card)| {
            is_core_candidate(id, card, favorite_ids)
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
        .filter(|(id, card)| is_core_candidate(id, card, favorite_ids) && card.memory().is_new())
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
            is_core_candidate(id, card, favorite_ids)
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

/// A phrase is tail-eligible when every non-particle token it references is
/// part of the known vocabulary pool. Grammatical particles (は, が, を, …)
/// attach to words rather than carrying standalone meaning, so they never
/// block eligibility. The pool spans the ENTIRE knowledge_set (built by
/// `collect_known_vocabulary_words`), not just lesson cards.
fn phrase_tail_eligible(phrase_id: &Ulid, known_pool: &HashSet<String>) -> bool {
    let Some(entry) = crate::dictionary::phrase::get_index_entry(phrase_id) else {
        return false;
    };
    entry.tokens().iter().all(|token| {
        crate::domain::grammar::is_grammatical_particle(token) || known_pool.contains(token)
    })
}

/// Bundles the immutable inputs driving tail-phrase selection. Grouping them
/// keeps `collect_phrase_cards` below the argument-count threshold and makes
/// the selection context explicit at the call site.
struct TailPhraseSelection<'a> {
    all_cards: &'a [(&'a Ulid, &'a StudyCard)],
    excluded_card_ids: &'a HashSet<Ulid>,
    used_phrase_ids: &'a HashSet<Ulid>,
    known_pool: &'a HashSet<String>,
    /// Remaining new-phrase allowance (already shared with the interleaved
    /// section). Due phrases do not consume it; new phrases decrement it
    /// greedily so a depleted budget halts further new-phrase admission.
    new_phrase_budget: &'a mut usize,
}

fn collect_phrase_cards<'a>(
    selection: &mut TailPhraseSelection<'a>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    let phrase_eligible = |id: &&Ulid, card: &&StudyCard| {
        let Card::Phrase(phrase_card) = card.card() else {
            return false;
        };
        !selection.excluded_card_ids.contains(id)
            && !selection.used_phrase_ids.contains(phrase_card.phrase_id())
            && phrase_tail_eligible(phrase_card.phrase_id(), selection.known_pool)
    };

    // `all_cards` is sorted by `next_review_date` asc upstream (see
    // `add_tail_phrases`), so filtering preserves the scheduling order without
    // an explicit secondary sort.
    let mut cap = PerWordCap::new(selection.known_pool);
    let mut phrase_cards: Vec<(&'a Ulid, &'a StudyCard)> = Vec::new();

    // Due phrases first — free of budget cost, but they still occupy per-word
    // cap slots so a frequent word cannot crowd out the tail through scheduling
    // pressure alone.
    for (id, card) in selection.all_cards.iter().copied() {
        if !phrase_eligible(&id, &card) || !card.memory().is_due() {
            continue;
        }
        if cap.try_admit(card) {
            phrase_cards.push((id, card));
        }
    }

    // New phrases are admitted only when both the budget AND the per-word cap
    // permit. The cap check runs before the decrement so the budget is never
    // spent on a phrase the cap would drop.
    for (id, card) in selection.all_cards.iter().copied() {
        if *selection.new_phrase_budget == 0 {
            break;
        }
        if !phrase_eligible(&id, &card) || !card.memory().is_new() {
            continue;
        }
        if !cap.try_admit(card) {
            continue;
        }
        phrase_cards.push((id, card));
        *selection.new_phrase_budget -= 1;
    }

    phrase_cards.truncate(PHRASE_MAX_PER_LESSON);
    phrase_cards
}

/// Streaming enforcer of `MAX_PHRASES_PER_WORD_IN_TAIL`. `try_admit` returns
/// `true` and reserves the phrase's known-word slots when the phrase still
/// fits the cap, `false` when at least one anchored word is already saturated.
/// Phrases are consumed in admission order (due before new) so the most
/// relevant phrase wins a word's slot when contention occurs.
struct PerWordCap<'a> {
    known_pool: &'a HashSet<String>,
    word_count: HashMap<&'static str, usize>,
}

impl<'a> PerWordCap<'a> {
    fn new(known_pool: &'a HashSet<String>) -> Self {
        Self {
            known_pool,
            word_count: HashMap::new(),
        }
    }

    fn try_admit(&mut self, card: &StudyCard) -> bool {
        let Card::Phrase(phrase_card) = card.card() else {
            return false;
        };
        let Some(entry) = crate::dictionary::phrase::get_index_entry(phrase_card.phrase_id())
        else {
            return false;
        };
        let over_cap = entry.tokens().iter().any(|token| {
            !crate::domain::grammar::is_grammatical_particle(token)
                && self.known_pool.contains(token)
                && self.word_count.get(token.as_str()).copied().unwrap_or(0)
                    >= MAX_PHRASES_PER_WORD_IN_TAIL
        });
        if over_cap {
            return false;
        }
        for token in entry.tokens() {
            if !crate::domain::grammar::is_grammatical_particle(token)
                && self.known_pool.contains(token)
            {
                *self.word_count.entry(token.as_str()).or_insert(0) += 1;
            }
        }
        true
    }
}

/// Interleaves phrase cards into the core stream so each phrase appears after
/// its anchor word with at least `gap` other cards between them. The invariant
/// `phrase_position > word_position` is preserved even when the lesson is too
/// short to honour the gap (remaining phrases flush at the end).
fn interleave_with_gap(
    core_cards: Vec<(Ulid, LessonCard)>,
    mut assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>>,
    gap: usize,
) -> Vec<(Ulid, LessonCard)> {
    let pending_phrases: usize = assignments.values().map(|v| v.len()).sum();
    let mut result: Vec<(Ulid, LessonCard)> =
        Vec::with_capacity(core_cards.len() + pending_phrases);
    let mut pending: Vec<(usize, (Ulid, LessonCard))> = Vec::new();

    for card in core_cards {
        let word_id = card.0;
        result.push(card);

        if let Some(phrases) = assignments.remove(&word_id) {
            let word_pos = result.len() - 1;
            for phrase in phrases {
                pending.push((word_pos + gap + 1, phrase));
            }
        }

        let mut deferred = Vec::with_capacity(pending.len());
        for (min_pos, phrase) in pending.drain(..) {
            if result.len() >= min_pos {
                result.push(phrase);
            } else {
                deferred.push((min_pos, phrase));
            }
        }
        pending = deferred;
    }

    for (_, phrase) in pending {
        result.push(phrase);
    }

    // Any assignment left in `assignments` had no matching core card and would
    // be silently dropped. Surface this as a programmer error in debug builds.
    debug_assert!(
        assignments.is_empty(),
        "interleave_with_gap dropped phrase assignments for words not present in core_cards"
    );

    result
}

/// Picks up to `INTERLEAVED_PHRASES_PER_WORD` phrase study cards for a single
/// anchor word. Due phrases win slots for free; new phrases fill the remainder
/// and each consumes one unit of the shared new-phrase budget.
fn collect_interleaved_phrases_for_word<'a>(
    word: &str,
    phrase_cards_by_id: &'a HashMap<Ulid, (&'a Ulid, &'a StudyCard)>,
    in_lesson: &'a HashSet<Ulid>,
    used_phrase_ids: &'a mut HashSet<Ulid>,
    phrase_new_budget: &'a mut usize,
) -> Vec<(Ulid, &'a StudyCard)> {
    let mut picker = InterleavePicker {
        phrase_cards_by_id,
        in_lesson,
        used_phrase_ids,
        phrase_new_budget,
    };
    let entries = crate::dictionary::phrase::get_phrases_by_token(word);
    let mut picked: Vec<(Ulid, &'a StudyCard)> = Vec::new();

    // `MemoryState::is_due` already implies `!is_new`, so due phrases are a
    // strict subset disjoint from the new-phrase pass below.
    picker.fill(&entries, &mut picked, |sc| sc.memory().is_due(), false);
    picker.fill(&entries, &mut picked, |sc| sc.memory().is_new(), true);

    picked
}

/// Shared selection state for the two interleaving passes (due then new) of a
/// single anchor word. Grouping the lookup inputs keeps `fill` below the
/// argument-count threshold and makes the pass context explicit.
struct InterleavePicker<'a> {
    phrase_cards_by_id: &'a HashMap<Ulid, (&'a Ulid, &'a StudyCard)>,
    in_lesson: &'a HashSet<Ulid>,
    used_phrase_ids: &'a mut HashSet<Ulid>,
    phrase_new_budget: &'a mut usize,
}

impl<'a> InterleavePicker<'a> {
    /// Appends eligible phrases to `picked` until `INTERLEAVED_PHRASES_PER_WORD`
    /// is reached or the budget runs out. `consume_budget` ties new-phrase
    /// consumption to the shared allowance (free for due phrases).
    fn fill<F>(
        &mut self,
        entries: &[&'static crate::dictionary::phrase::IndexEntry],
        picked: &mut Vec<(Ulid, &'a StudyCard)>,
        memory_predicate: F,
        consume_budget: bool,
    ) where
        F: Fn(&StudyCard) -> bool,
    {
        for entry in entries {
            if picked.len() >= INTERLEAVED_PHRASES_PER_WORD {
                break;
            }
            if consume_budget && *self.phrase_new_budget == 0 {
                break;
            }
            let pid = entry.id();
            if self.used_phrase_ids.contains(pid) {
                continue;
            }
            let Some(&(card_id, sc)) = self.phrase_cards_by_id.get(pid) else {
                continue;
            };
            if self.in_lesson.contains(card_id) {
                continue;
            }
            if !memory_predicate(sc) {
                continue;
            }
            picked.push((*card_id, sc));
            self.used_phrase_ids.insert(*pid);
            if consume_budget {
                *self.phrase_new_budget -= 1;
            }
        }
    }
}

fn build_phrase_assignments(
    targets: &[(&Ulid, String)],
    phrase_cards_by_id: &HashMap<Ulid, (&Ulid, &StudyCard)>,
    in_lesson: &HashSet<Ulid>,
    used_phrase_ids: &mut HashSet<Ulid>,
    phrase_new_budget: &mut usize,
    generator: &mut LessonViewGenerator,
) -> HashMap<Ulid, Vec<(Ulid, LessonCard)>> {
    let mut assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>> = HashMap::new();

    for (word_card_id, word_text) in targets {
        if assignments.contains_key(word_card_id) {
            continue;
        }
        let picked = collect_interleaved_phrases_for_word(
            word_text,
            phrase_cards_by_id,
            in_lesson,
            used_phrase_ids,
            phrase_new_budget,
        );
        if picked.is_empty() {
            continue;
        }
        let lesson_cards = picked
            .into_iter()
            .map(|(card_id, sc)| {
                let view = generator.apply_view(sc, sc.is_new(), &mut rand::rng());
                (card_id, LessonCard::new(view, false))
            })
            .collect();
        assignments.insert(**word_card_id, lesson_cards);
    }

    assignments
}

/// Inserts up to `INTERLEAVED_PHRASES_PER_WORD` phrases per anchor word into the
/// core section. Anchor words are new/in-progress vocab; if none yield phrases,
/// known vocab is used as a fallback. Interleaved phrases become part of
/// `core_count` and are excluded from the tail via `used_phrase_ids`.
pub(crate) fn add_interleaved_phrases(
    mut lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    used_phrase_ids: &mut HashSet<Ulid>,
    phrase_new_budget: &mut usize,
) -> LessonData {
    let core_count = lesson_data.core_count;
    if core_count == 0 {
        return lesson_data;
    }

    let phrase_cards_by_id: HashMap<Ulid, (&Ulid, &StudyCard)> = knowledge_set
        .study_cards()
        .iter()
        .filter_map(|(id, sc)| match sc.card() {
            Card::Phrase(pc) => Some((*pc.phrase_id(), (id, sc))),
            _ => None,
        })
        .collect();

    let in_lesson: HashSet<Ulid> = lesson_data.cards.iter().map(|(id, _)| *id).collect();

    let core_vocab: Vec<(Ulid, String)> = lesson_data.cards[..core_count]
        .iter()
        .filter_map(|(id, lc)| match lc.card() {
            Card::Vocabulary(v) => Some((*id, v.word().text().to_string())),
            _ => None,
        })
        .collect();

    // Interleaving exists to reinforce vocab that is still being learned — so
    // the target set is everything that is NOT a stable known card. This
    // intentionally includes high-difficulty cards (the original motivation
    // for interleaving).
    let (non_known, known) = core_vocab
        .iter()
        .map(|(id, word)| (id, word.clone()))
        .partition::<Vec<_>, _>(|(id, _)| {
            knowledge_set
                .get_card(**id)
                .map(|sc| !sc.memory().is_known_card())
                .unwrap_or(false)
        });

    let mut generator = LessonViewGenerator::new(knowledge_set);

    let mut assignments = build_phrase_assignments(
        &non_known,
        &phrase_cards_by_id,
        &in_lesson,
        used_phrase_ids,
        phrase_new_budget,
        &mut generator,
    );

    if assignments.is_empty() && !known.is_empty() {
        assignments = build_phrase_assignments(
            &known,
            &phrase_cards_by_id,
            &in_lesson,
            used_phrase_ids,
            phrase_new_budget,
            &mut generator,
        );
    }

    if assignments.is_empty() {
        return lesson_data;
    }

    let core_cards = std::mem::take(&mut lesson_data.cards);
    lesson_data.cards = interleave_with_gap(core_cards, assignments, INTERLEAVING_GAP);
    lesson_data.core_count = lesson_data.cards.len();
    lesson_data
}

/// Appends the tail phrases after the core section. Tail phrases only use
/// vocabulary the learner has already met anywhere in the knowledge_set (not
/// just the current lesson), and share the remaining new-phrase budget with
/// the interleaved section (due phrases are free).
pub(crate) fn add_tail_phrases(
    mut lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    used_phrase_ids: &HashSet<Ulid>,
    phrase_new_budget: &mut usize,
) -> LessonData {
    let mut all_cards = knowledge_set.study_cards().iter().collect::<Vec<_>>();
    all_cards.sort_by_key(|(_, card)| card.memory().next_review_date());

    let excluded: HashSet<Ulid> = lesson_data.cards.iter().map(|(id, _)| *id).collect();

    let known_pool =
        super::collect_known_vocabulary_words(knowledge_set.study_cards().values(), true);

    let mut selection = TailPhraseSelection {
        all_cards: &all_cards,
        excluded_card_ids: &excluded,
        used_phrase_ids,
        known_pool: &known_pool,
        new_phrase_budget: phrase_new_budget,
    };
    let phrase_cards = collect_phrase_cards(&mut selection);

    if phrase_cards.is_empty() {
        return lesson_data;
    }

    let mut generator = LessonViewGenerator::new(knowledge_set);
    let phrase_lessons: Vec<(Ulid, LessonCard)> = phrase_cards
        .iter()
        .map(|(card_id, sc)| {
            let view = generator.apply_view(sc, sc.is_new(), &mut rand::rng());
            (**card_id, LessonCard::new(view, false))
        })
        .collect();

    lesson_data.cards.extend(phrase_lessons);
    lesson_data
}

fn build_selected_ids(
    selected_cards: &[(&Ulid, &StudyCard)],
    favorite_cards: &[(&Ulid, &StudyCard)],
) -> HashSet<Ulid> {
    let selected_ids: HashSet<_> = selected_cards.iter().map(|(id, _)| **id).collect();
    let favorite_ids: HashSet<_> = favorite_cards.iter().map(|(id, _)| **id).collect();
    selected_ids.union(&favorite_ids).copied().collect()
}

fn build_core_lesson_cards(
    favorite_cards: &[(&Ulid, &StudyCard)],
    selected_cards: &[(&Ulid, &StudyCard)],
    padding_cards: &[(&Ulid, &StudyCard)],
    knowledge_set: &KnowledgeSet,
) -> Vec<(Ulid, LessonCard)> {
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

    let mut result = favorite_lessons;
    result.extend(selected_lessons);
    result.extend(padding_lessons);
    result
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
    use crate::domain::knowledge::LessonCardView;
    use crate::domain::knowledge::{PhraseCard, VocabularyCard};
    use crate::domain::value_objects::Question;
    use crate::domain::{RateMode, Rating};

    fn vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn phrase_card(phrase_id: Ulid) -> Card {
        Card::Phrase(PhraseCard::new_test_with_id(phrase_id))
    }

    fn make_study_card(card: Card) -> (Ulid, StudyCard) {
        (Ulid::new(), StudyCard::new(card))
    }

    // --- Tail phrase selection (Slice-3) ---
    //
    // The phrase index is a process-wide `OnceLock`; only one index can live in
    // a test binary. These tests reuse the exact 8-phrase fixture also used by
    // `journeys/phrase.rs` and `seed_ready_phrases.rs` so they hold regardless
    // of which module wins the initialization race.

    static PHRASE_INDEX_INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

    fn phrase_id_hello() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID")
    }

    fn phrase_id_bye() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HK").expect("valid ULID")
    }

    fn phrase_id_morning() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HN").expect("valid ULID")
    }

    fn phrase_id_thanks() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HM").expect("valid ULID")
    }

    fn phrase_id_particle() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HP").expect("valid ULID")
    }

    fn phrase_id_extra1() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HQ").expect("valid ULID")
    }

    fn phrase_id_extra2() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HR").expect("valid ULID")
    }

    fn phrase_id_independent() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HS").expect("valid ULID")
    }

    fn ensure_test_phrase_index() {
        PHRASE_INDEX_INIT.get_or_init(|| {
            if crate::dictionary::phrase::is_phrases_loaded() {
                return;
            }
            let index_json = r#"{"v":1,"h":"test","total":8,"phrases":[
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["test","hello"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["test","bye"],"c":0,"g":["01KJ9AVWBGC2BT0DMFPDYYFEWB"]},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HN","t":["test","morning"],"c":0,"g":["01KJ9AVWBGC2BT0DMFPDYYFEWB","01G00000000000000024000000"]},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HM","t":["test","thanks"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HP","t":["test","は"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HQ","t":["hello","extra1"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HR","t":["hello","extra2"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HS","t":["alpha","beta"],"c":0}
            ]}"#;
            crate::dictionary::phrase::init_phrase_index(index_json)
                .expect("Failed to init test phrase index");
        });
    }

    fn full_known_set() -> HashSet<String> {
        ["test", "hello", "bye", "morning", "thanks"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn phrase_tail_eligible_true_when_all_tokens_known() {
        ensure_test_phrase_index();

        let known = full_known_set();
        assert!(phrase_tail_eligible(&phrase_id_hello(), &known));
        assert!(phrase_tail_eligible(&phrase_id_bye(), &known));
    }

    #[test]
    fn phrase_tail_eligible_false_when_token_missing() {
        ensure_test_phrase_index();

        let partial: HashSet<String> = ["test", "hello"].iter().map(|s| s.to_string()).collect();
        assert!(!phrase_tail_eligible(&phrase_id_bye(), &partial));
    }

    #[test]
    fn phrase_tail_eligible_false_missing_index_entry() {
        ensure_test_phrase_index();

        let known = full_known_set();
        assert!(!phrase_tail_eligible(&Ulid::new(), &known));
    }

    // --- Prove-It test suite (Slice-1, RED) ---
    //
    // These tests pin the new tail-eligibility and interleaving contracts
    // introduced to fix the post-PR-#188 regression. They drive the Slice-2
    // fix and must remain green afterwards. The contract:
    //   * Tail eligibility ignores grammatical particles and draws the
    //     known-pool from the ENTIRE knowledge_set (not just lesson cards).
    //   * Tail per-word cap is 1 (down from 2).
    //   * Interleaved anchor set is `!is_known_card()` — high-difficulty
    //     vocab is intentionally included.
    //   * There is no global interleaved cap (only per-word cap).

    #[test]
    fn phrase_tail_eligible_ignores_grammatical_particle() {
        ensure_test_phrase_index();

        let known: HashSet<String> = ["test"].iter().map(|s| s.to_string()).collect();
        assert!(
            phrase_tail_eligible(&phrase_id_particle(), &known),
            "particle token は must be ignored when judging tail eligibility"
        );
    }

    #[test]
    fn phrase_tail_eligible_rejects_unknown_non_particle_token() {
        ensure_test_phrase_index();

        let known: HashSet<String> = ["hello"].iter().map(|s| s.to_string()).collect();
        assert!(
            !phrase_tail_eligible(&phrase_id_extra1(), &known),
            "unknown non-particle token must disqualify the phrase"
        );
    }

    #[test]
    fn phrase_tail_eligible_all_known_vocab_eligible() {
        ensure_test_phrase_index();

        let known: HashSet<String> = ["test", "hello"].iter().map(|s| s.to_string()).collect();
        assert!(
            phrase_tail_eligible(&phrase_id_hello(), &known),
            "phrase whose tokens are all known must be eligible"
        );
    }

    /// Tail eligibility draws its known-pool from the ENTIRE knowledge_set, not
    /// just the lesson cards. A phrase anchored to vocab that lives in the
    /// knowledge_set but is NOT in the current lesson's core must still be
    /// tail-eligible. This is the core regression fix.
    #[test]
    fn phrase_tail_eligible_uses_entire_knowledge_set() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let test_sc = ks.create_card(vocab_card("test")).expect("create test");
        ks.mark_card_as_known(*test_sc.card_id())
            .expect("mark test known");
        let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
        ks.mark_card_as_known(*hello_sc.card_id())
            .expect("mark hello known");
        ks.create_card(Card::Phrase(
            PhraseCard::new_test_with_id(phrase_id_hello()),
        ))
        .expect("create phrase");

        let lesson = LessonData {
            cards: vec![(Ulid::new(), lesson_card_for(vocab_card("猫")))],
            core_count: 1,
        };

        let used = HashSet::new();
        let mut budget = 5;
        let result = add_tail_phrases(lesson, &ks, &used, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.contains(&phrase_id_hello()),
            "phrase anchored to known vocab outside the lesson core must enter the tail"
        );
    }

    #[test]
    fn tail_phrases_cap_one_per_word() {
        ensure_test_phrase_index();

        let owned = make_phrase_study_cards(&[
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ]);
        let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

        let known = full_known_set();
        let empty_used = HashSet::new();
        let mut budget = 20;
        let mut selection = tail_selection(&all_cards, &known, &empty_used, &mut budget);
        let result = collect_phrase_cards(&mut selection);

        assert_eq!(
            result.len(),
            1,
            "Tail phrases sharing a word should be capped at 1 (MAX_PHRASES_PER_WORD_IN_TAIL=1), got {}",
            result.len()
        );
    }

    /// The shared new-phrase budget (interleaved + tail) caps the number of
    /// new phrases added to the lesson. Due phrases are free.
    #[test]
    fn tail_phrases_new_fill_respects_shared_budget() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        for word in ["test", "hello", "bye", "morning", "thanks"] {
            let sc = ks.create_card(vocab_card(word)).expect("create vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ] {
            ks.create_card(Card::Phrase(PhraseCard::new_test_with_id(pid)))
                .expect("create phrase");
        }

        let lesson = LessonData {
            cards: vec![(Ulid::new(), lesson_card_for(vocab_card("猫")))],
            core_count: 1,
        };

        let used = HashSet::new();
        let mut budget = 2;
        let result = add_tail_phrases(lesson, &ks, &used, &mut budget);
        let new_phrases_count = lesson_phrase_ids(&result).len();

        assert!(
            new_phrases_count <= 2,
            "new tail phrases must respect the shared budget: {new_phrases_count} > 2"
        );

        let used = HashSet::new();
        let mut zero_budget = 0;
        let zero_budget_result = add_tail_phrases(
            LessonData {
                cards: vec![(Ulid::new(), lesson_card_for(vocab_card("猫")))],
                core_count: 1,
            },
            &ks,
            &used,
            &mut zero_budget,
        );
        assert_eq!(
            lesson_phrase_ids(&zero_budget_result).len(),
            0,
            "with budget=0 no new tail phrases may be added"
        );
    }

    /// Due phrases precede new phrases in the tail (they are scheduled for
    /// review), preserving the natural scheduling order.
    #[test]
    fn tail_phrases_preserve_due_then_new_order() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        // All tokens referenced by the eligible phrases are known so neither
        // phrase gets filtered out; the two phrases deliberately anchor on
        // disjoint tokens ("alpha"/"beta" vs "test"/"hello") so the per-word
        // cap (MAX_PHRASES_PER_WORD_IN_TAIL=1) does not silently drop one.
        for word in ["test", "hello", "alpha", "beta"] {
            let sc = ks.create_card(vocab_card(word)).expect("create vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }

        let due_sc = ks
            .create_card(Card::Phrase(PhraseCard::new_test_with_id(
                phrase_id_independent(),
            )))
            .expect("create due phrase");
        ks.mark_card_as_known(*due_sc.card_id())
            .expect("mark phrase due");

        ks.create_card(Card::Phrase(
            PhraseCard::new_test_with_id(phrase_id_hello()),
        ))
        .expect("create new phrase");

        let lesson = LessonData {
            cards: vec![(Ulid::new(), lesson_card_for(vocab_card("猫")))],
            core_count: 1,
        };

        let used = HashSet::new();
        let core_len = lesson.cards.len();
        let mut budget = 5;
        let result = add_tail_phrases(lesson, &ks, &used, &mut budget);
        let phrase_sequence: Vec<Option<Ulid>> = result
            .cards
            .iter()
            .skip(core_len)
            .map(|(_, lc)| lesson_phrase_id(lc))
            .collect();

        let due_pos = phrase_sequence
            .iter()
            .position(|id| id == &Some(phrase_id_independent()));
        let new_pos = phrase_sequence
            .iter()
            .position(|id| id == &Some(phrase_id_hello()));

        let due_pos = due_pos.expect("due phrase must be present in the tail");
        let new_pos = new_pos.expect("new phrase must be present in the tail");
        assert!(
            due_pos < new_pos,
            "due phrase must precede new phrase in the tail: due={due_pos}, new={new_pos}"
        );
    }

    /// High-difficulty vocab is now a valid interleaving target (the original
    /// purpose of interleaving): a phrase anchored to an HD word must appear.
    #[test]
    fn interleaved_phrases_target_high_difficulty() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let bye_sc = ks.create_card(vocab_card("bye")).expect("create bye");
        for word in ["test", "hello"] {
            ks.create_card(vocab_card(word))
                .expect("create known vocab");
        }
        ks.create_card(phrase_card(phrase_id_bye()))
            .expect("create phrase");

        for _ in 0..3 {
            ks.rate_card(*bye_sc.card_id(), Rating::Again, RateMode::ShortTerm)
                .expect("rate bye");
        }

        let bye_id = *bye_sc.card_id();
        assert!(
            ks.get_card(bye_id).unwrap().memory().is_high_difficulty(),
            "fixture sanity: bye should be high difficulty"
        );

        let lesson = LessonData {
            cards: vec![(bye_id, lesson_card_for(vocab_card("bye")))],
            core_count: 1,
        };

        let mut used = HashSet::new();
        let mut budget = 5;
        let result = add_interleaved_phrases(lesson, &ks, &mut used, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.contains(&phrase_id_bye()),
            "high-difficulty anchor must receive an interleaved phrase"
        );
    }

    /// No global interleaved cap exists: two anchor words each yield
    /// `INTERLEAVED_PHRASES_PER_WORD` phrases, summing to 4 — proving the
    /// absence of a hidden total ceiling.
    #[test]
    fn interleaved_phrases_no_total_cap_multi_anchor() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let test_sc = ks.create_card(vocab_card("test")).expect("create test");
        let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_extra1(),
            phrase_id_extra2(),
        ] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        let test_id = *test_sc.card_id();
        let hello_id = *hello_sc.card_id();
        let lesson = LessonData {
            cards: vec![
                (test_id, lesson_card_for(vocab_card("test"))),
                (hello_id, lesson_card_for(vocab_card("hello"))),
            ],
            core_count: 2,
        };

        let mut used = HashSet::new();
        let mut budget = 10;
        let result = add_interleaved_phrases(lesson, &ks, &mut used, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.len() >= INTERLEAVED_PHRASES_PER_WORD * 2,
            "two anchors × {} phrases each (no total cap) should yield ≥ {}, got {}",
            INTERLEAVED_PHRASES_PER_WORD,
            INTERLEAVED_PHRASES_PER_WORD * 2,
            phrases.len()
        );
    }

    fn make_phrase_study_cards(phrase_ids: &[Ulid]) -> Vec<(Ulid, StudyCard)> {
        phrase_ids
            .iter()
            .map(|pid| make_study_card(phrase_card(*pid)))
            .collect()
    }

    fn selected_phrase_ids(cards: &[(&Ulid, &StudyCard)]) -> Vec<Ulid> {
        cards
            .iter()
            .filter_map(|(_, sc)| match sc.card() {
                Card::Phrase(p) => Some(*p.phrase_id()),
                _ => None,
            })
            .collect()
    }

    fn empty_ulid_set() -> &'static HashSet<Ulid> {
        static EMPTY: std::sync::OnceLock<HashSet<Ulid>> = std::sync::OnceLock::new();
        EMPTY.get_or_init(HashSet::new)
    }

    fn tail_selection<'a>(
        all_cards: &'a [(&'a Ulid, &'a StudyCard)],
        known: &'a HashSet<String>,
        used: &'a HashSet<Ulid>,
        budget: &'a mut usize,
    ) -> TailPhraseSelection<'a> {
        TailPhraseSelection {
            all_cards,
            excluded_card_ids: empty_ulid_set(),
            used_phrase_ids: used,
            known_pool: known,
            new_phrase_budget: budget,
        }
    }

    #[test]
    fn tail_phrases_contain_only_known_words() {
        ensure_test_phrase_index();

        // One eligible phrase plus one unknown phrase id. The eligible phrase
        // has all-known tokens; the unknown phrase id is absent from the index,
        // so `phrase_tail_eligible` rejects it via `get_index_entry` returning
        // None.
        let owned = make_phrase_study_cards(&[phrase_id_hello(), Ulid::new()]);
        let unknown_phrase = match owned[1].1.card() {
            Card::Phrase(p) => *p.phrase_id(),
            _ => unreachable!("second card is a phrase card"),
        };
        let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

        let known = full_known_set();
        let empty_used = HashSet::new();
        let mut budget = 20;
        let mut selection = tail_selection(&all_cards, &known, &empty_used, &mut budget);
        let result = collect_phrase_cards(&mut selection);

        let selected = selected_phrase_ids(&result);
        assert!(
            selected.contains(&phrase_id_hello()),
            "phrase with all-known tokens should be selected"
        );
        assert!(
            !selected.contains(&unknown_phrase),
            "phrase whose phrase_id is absent from the index must be excluded"
        );
    }

    #[test]
    fn tail_phrases_exclude_used_phrase_ids() {
        ensure_test_phrase_index();

        let owned = make_phrase_study_cards(&[phrase_id_hello(), phrase_id_bye()]);
        let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

        let known = full_known_set();
        let mut used = HashSet::new();
        used.insert(phrase_id_hello());

        let mut budget = 20;
        let mut selection = tail_selection(&all_cards, &known, &used, &mut budget);
        let result = collect_phrase_cards(&mut selection);

        let selected = selected_phrase_ids(&result);
        assert!(
            !selected.contains(&phrase_id_hello()),
            "phrase already used by interleaving must not reappear in the tail"
        );
        assert!(selected.contains(&phrase_id_bye()));
    }

    #[test]
    fn init_phrase_index_loads_entries() {
        // The CDN loader (`init_phrase_index_from_cdn`) is process-wide and
        // mutually exclusive with the 8-phrase fixture used across the lib test
        // binary, so loading the real 156k-entry index here would win the
        // OnceLock race and break fixture-based tests. We instead verify the
        // helper is exported and callable, and assert the index contract
        // (entries become retrievable) via the safe fixture path.
        let _helper: fn() = crate::use_cases::init_phrase_index_from_cdn;

        ensure_test_phrase_index();

        assert!(
            crate::dictionary::phrase::is_phrases_loaded(),
            "phrase index should be loaded after init"
        );
        assert!(
            crate::dictionary::phrase::iter_index_entries().is_some(),
            "iter_index_entries should yield entries once the index is loaded"
        );
        assert!(
            crate::dictionary::phrase::get_phrases_by_token("test")
                .iter()
                .any(|e| !e.tokens().is_empty()),
            "get_phrases_by_token should resolve known fixture tokens to entries"
        );
    }

    // --- Interleaving algorithm (H1) ---
    //
    // These tests pin the invariants of the phrase-interleaving pipeline. They
    // intentionally span three levels: the pure layout primitive
    // `interleave_with_gap`, the mid-level orchestrator `add_interleaved_phrases`
    // / `collect_interleaved_phrases_for_word`, and the full `cards_to_lesson`
    // pipeline. Lower levels isolate a single invariant; the pipeline tests
    // guard the integration of the shared budget and `used_phrase_ids` set.

    fn lesson_card_for(card: Card) -> LessonCard {
        LessonCard::new(LessonCardView::Normal(card), false)
    }

    fn lesson_phrase_id(lc: &LessonCard) -> Option<Ulid> {
        match lc.card() {
            Card::Phrase(p) => Some(*p.phrase_id()),
            _ => None,
        }
    }

    fn phrase_cards_map(owned: &[(Ulid, StudyCard)]) -> HashMap<Ulid, (&Ulid, &StudyCard)> {
        owned
            .iter()
            .filter_map(|(id, sc)| match sc.card() {
                Card::Phrase(p) => Some((*p.phrase_id(), (id, sc))),
                _ => None,
            })
            .collect()
    }

    fn lesson_phrase_ids(lesson: &LessonData) -> HashSet<Ulid> {
        lesson
            .cards
            .iter()
            .filter_map(|(_, lc)| lesson_phrase_id(lc))
            .collect()
    }

    /// A phrase anchored to a word must land strictly after it, with at least
    /// `gap` intervening cards when the core is long enough to honour the gap.
    #[test]
    fn interleaved_phrases_placed_after_word_with_gap() {
        let word_id = Ulid::new();
        let phrase_card_id = Ulid::new();

        let mut core_cards: Vec<(Ulid, LessonCard)> = (0..5)
            .map(|_| (Ulid::new(), lesson_card_for(vocab_card("filler"))))
            .collect();
        core_cards[2] = (word_id, lesson_card_for(vocab_card("anchor")));

        let assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>> = [(
            word_id,
            vec![(phrase_card_id, lesson_card_for(phrase_card(Ulid::new())))],
        )]
        .into_iter()
        .collect();

        let result = interleave_with_gap(core_cards, assignments, INTERLEAVING_GAP);

        let word_pos = result
            .iter()
            .position(|(id, _)| *id == word_id)
            .expect("anchor word present in output");
        let phrase_pos = result
            .iter()
            .position(|(id, _)| *id == phrase_card_id)
            .expect("interleaved phrase present in output");

        assert!(
            phrase_pos > word_pos,
            "phrase must follow its anchor word: {phrase_pos} <= {word_pos}"
        );
        assert!(
            phrase_pos > word_pos + INTERLEAVING_GAP,
            "core had room for the gap: phrase_pos {phrase_pos} should be > {}",
            word_pos + INTERLEAVING_GAP
        );
    }

    /// No single anchor word may pull more than `INTERLEAVED_PHRASES_PER_WORD`
    /// phrases into the core, even when many eligible phrases share its token.
    #[test]
    fn interleaved_phrases_max_two_per_word() {
        ensure_test_phrase_index();

        let owned = make_phrase_study_cards(&[
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ]);
        let map = phrase_cards_map(&owned);

        let in_lesson = HashSet::new();
        let mut used = HashSet::new();
        let mut budget = 10;

        let picked =
            collect_interleaved_phrases_for_word("test", &map, &in_lesson, &mut used, &mut budget);

        assert!(
            picked.len() <= INTERLEAVED_PHRASES_PER_WORD,
            "per-word cap violated: {} > {INTERLEAVED_PHRASES_PER_WORD}",
            picked.len()
        );
        assert_eq!(
            picked.len(),
            INTERLEAVED_PHRASES_PER_WORD,
            "four eligible phrases with a deep budget should fill the cap exactly"
        );
    }

    /// Interleaving falls back to known vocab ONLY when no non-known (new /
    /// in-progress / high-difficulty) anchor yields a phrase. The fallback
    /// exists to keep phrases flowing on a fully-mastered lesson — the target
    /// filter `!is_known_card()` covers new, in-progress and high-difficulty
    /// vocab alike (see `interleaved_phrases_target_high_difficulty`), so this
    /// test exercises the residual branch where every core vocab is known.
    #[test]
    fn interleaved_phrases_fallback_to_known() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
        let test_sc = ks.create_card(vocab_card("test")).expect("create test");
        for pid in [phrase_id_hello(), phrase_id_bye()] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        ks.mark_card_as_known(*hello_sc.card_id())
            .expect("mark hello known");
        ks.mark_card_as_known(*test_sc.card_id())
            .expect("mark test known");

        let hello_id = *hello_sc.card_id();
        let test_id = *test_sc.card_id();
        assert!(ks.get_card(hello_id).unwrap().memory().is_known_card());
        assert!(ks.get_card(test_id).unwrap().memory().is_known_card());

        let lesson = LessonData {
            cards: vec![
                (hello_id, lesson_card_for(vocab_card("hello"))),
                (test_id, lesson_card_for(vocab_card("test"))),
            ],
            core_count: 2,
        };

        let mut used = HashSet::new();
        let mut budget = 5;
        let result = add_interleaved_phrases(lesson, &ks, &mut used, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.contains(&phrase_id_hello()),
            "fallback should anchor a phrase to a known word when no target vocab yields phrases"
        );
    }

    /// A phrase consumed by the interleaved section (tracked via
    /// `used_phrase_ids`) must never reappear in the tail, and vice versa.
    #[test]
    fn interleaved_phrases_no_overlap_with_tail() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        ks.create_card(vocab_card("hello")).expect("create hello");
        for word in ["test", "bye", "morning", "thanks"] {
            let sc = ks
                .create_card(vocab_card(word))
                .expect("create known vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        let lesson = ks.cards_to_lesson(1, &JlptContent::new(), JapaneseLevel::N5);

        let core_count = lesson.core_count;
        let interleaved_ids: HashSet<Ulid> = lesson.cards[..core_count]
            .iter()
            .filter_map(|(_, lc)| lesson_phrase_id(lc))
            .collect();
        let tail_ids: HashSet<Ulid> = lesson.cards[core_count..]
            .iter()
            .filter_map(|(_, lc)| lesson_phrase_id(lc))
            .collect();

        assert!(
            !interleaved_ids.is_empty(),
            "scenario should produce at least one interleaved phrase"
        );
        let overlap: Vec<_> = interleaved_ids.intersection(&tail_ids).copied().collect();
        assert!(
            overlap.is_empty(),
            "interleaved and tail must not share phrase ids: {overlap:?}"
        );
    }

    /// `core_count` is recomputed after interleaving, so every phrase placed in
    /// the core section counts as core; the tail only starts after it.
    #[test]
    fn interleaved_phrases_core_count_includes_them() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        ks.create_card(vocab_card("hello")).expect("create hello");
        for word in ["test", "bye", "morning", "thanks"] {
            let sc = ks
                .create_card(vocab_card(word))
                .expect("create known vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        let lesson = ks.cards_to_lesson(1, &JlptContent::new(), JapaneseLevel::N5);
        let core_count = lesson.core_count;

        let interleaved_in_core = lesson.cards[..core_count]
            .iter()
            .filter(|(_, lc)| lesson_phrase_id(lc).is_some())
            .count();

        assert!(
            interleaved_in_core >= 1,
            "at least one interleaved phrase should sit inside the core section"
        );
        assert!(core_count >= interleaved_in_core);
        for (_, lc) in &lesson.cards[core_count..] {
            assert!(
                lesson_phrase_id(lc).is_some(),
                "everything past core_count must be a (tail) phrase card"
            );
        }
    }

    /// On a lesson too short to honour the gap, the phrase is flushed at the
    /// end. The only inviolable ordering rule — phrase after its anchor — holds.
    #[test]
    fn interleaved_phrases_small_lesson_degrades_gap() {
        let word_id = Ulid::new();
        let phrase_card_id = Ulid::new();

        let core_cards: Vec<(Ulid, LessonCard)> =
            [(word_id, lesson_card_for(vocab_card("solo")))].to_vec();
        let assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>> = [(
            word_id,
            vec![(phrase_card_id, lesson_card_for(phrase_card(Ulid::new())))],
        )]
        .into_iter()
        .collect();

        let result = interleave_with_gap(core_cards, assignments, INTERLEAVING_GAP);

        let word_pos = result
            .iter()
            .position(|(id, _)| *id == word_id)
            .expect("anchor present");
        let phrase_pos = result
            .iter()
            .position(|(id, _)| *id == phrase_card_id)
            .expect("flushed phrase present");

        assert_eq!(result.len(), 2);
        assert!(phrase_pos > word_pos);
        assert!(
            phrase_pos < word_pos + INTERLEAVING_GAP + 1,
            "gap should degrade on a tiny core, but ordering must survive"
        );
    }

    /// The new-phrase budget is shared between the interleaved and tail
    /// sections: together they may not exceed it. Due phrases are free.
    #[test]
    fn interleaved_and_tail_respect_shared_phrase_new_budget() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        ks.create_card(vocab_card("hello")).expect("create hello");
        for word in ["test", "bye", "morning", "thanks"] {
            let sc = ks
                .create_card(vocab_card(word))
                .expect("create known vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        let daily_new_limit = 1;
        let budget = compute_phrase_new_budget(daily_new_limit, 0);
        assert_eq!(budget, 2, "fixture sanity: PHRASE_NEW_RATIO=2");

        let lesson = ks.cards_to_lesson(daily_new_limit, &JlptContent::new(), JapaneseLevel::N5);

        let new_phrases_in_lesson = lesson
            .cards
            .iter()
            .filter(|(_, lc)| lesson_phrase_id(lc).is_some())
            .filter(|(id, _)| ks.get_card(*id).is_some_and(|sc| sc.memory().is_new()))
            .count();

        assert!(
            new_phrases_in_lesson <= budget,
            "new phrases (interleaved + tail) must respect the shared budget: {new_phrases_in_lesson} > {budget}"
        );
        assert!(
            new_phrases_in_lesson >= 1,
            "scenario should place at least one new phrase"
        );
    }

    /// Due phrases enter the interleaved section for free: they do not decrement
    /// `phrase_new_budget`, which stays reserved for new phrases.
    #[test]
    fn interleaved_phrases_due_do_not_consume_new_budget() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let hello_sc = ks
            .create_card(phrase_card(phrase_id_hello()))
            .expect("create hello");
        for pid in [phrase_id_bye(), phrase_id_morning(), phrase_id_thanks()] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        // mark_card_as_known schedules next_review in the past, so the phrase
        // becomes due (and not new) without going through a timed review cycle.
        ks.mark_card_as_known(*hello_sc.card_id())
            .expect("mark hello due");
        let hello_card = ks.get_card(*hello_sc.card_id()).unwrap();
        assert!(hello_card.memory().is_due());
        assert!(!hello_card.memory().is_new());

        let owned: Vec<(Ulid, StudyCard)> = ks
            .study_cards()
            .iter()
            .map(|(id, sc)| (*id, sc.clone()))
            .collect();
        let map = phrase_cards_map(&owned);

        let in_lesson = HashSet::new();
        let mut used = HashSet::new();
        let mut budget = 2;
        let initial_budget = budget;

        let picked =
            collect_interleaved_phrases_for_word("test", &map, &in_lesson, &mut used, &mut budget);

        let picked_due_hello = picked.iter().any(|(id, _)| *id == *hello_sc.card_id());
        assert!(
            picked_due_hello,
            "due phrase must be picked for free by the due pass"
        );

        let new_picked = picked.iter().filter(|(_, sc)| sc.memory().is_new()).count();
        assert_eq!(
            budget,
            initial_budget - new_picked,
            "only new phrases consume budget; the due phrase must be free"
        );
    }
}
