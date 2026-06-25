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
pub(crate) const MAX_PHRASES_PER_WORD_IN_TAIL: usize = 2;

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
    cards: impl IntoIterator<Item = (&'a Ulid, &'a StudyCard)>,
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

/// Tail phrases only reinforce already-mastered lesson vocab: a word qualifies
/// when its study card has reached a stable memory state (`is_known_card`),
/// i.e. stability above the known threshold and not flagged high-difficulty.
fn extract_known_lesson_words<'a>(
    cards: impl IntoIterator<Item = (&'a Ulid, &'a StudyCard)>,
) -> HashSet<String> {
    let mut words = HashSet::new();

    for (_, study_card) in cards {
        if !study_card.memory().is_known_card() {
            continue;
        }
        if let Card::Vocabulary(vocab) = study_card.card() {
            words.insert(vocab.word().text().to_string());
        }
    }

    words
}

/// A phrase is tail-eligible only when every token it references is a lesson
/// word the user has already mastered. This excludes phrases that lean on
/// newly-introduced or unstable vocabulary, which get reinforced via the
/// interleaved section instead.
fn phrase_tokens_all_known(phrase_id: &Ulid, known_lesson_words: &HashSet<String>) -> bool {
    let Some(entry) = crate::dictionary::phrase::get_index_entry(phrase_id) else {
        return false;
    };
    entry
        .tokens()
        .iter()
        .all(|token| known_lesson_words.contains(token))
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

/// Bundles the immutable inputs driving tail-phrase selection. Grouping them
/// keeps `collect_phrase_cards` below the argument-count threshold and makes
/// the selection context explicit at the call site.
struct TailPhraseSelection<'a> {
    all_cards: &'a [(&'a Ulid, &'a StudyCard)],
    excluded_card_ids: &'a HashSet<Ulid>,
    used_phrase_ids: &'a HashSet<Ulid>,
    known_lesson_words: &'a HashSet<String>,
    lesson_words: &'a HashSet<String>,
    lesson_grammar_ids: &'a HashSet<Ulid>,
    /// Remaining new-phrase allowance (already shared with the interleaved
    /// section). Due phrases do not consume it.
    new_phrase_budget: usize,
}

fn collect_phrase_cards<'a>(selection: &TailPhraseSelection<'a>) -> Vec<(&'a Ulid, &'a StudyCard)> {
    let phrase_new_remaining = selection.new_phrase_budget;

    let phrase_eligible = |id: &&Ulid, card: &&StudyCard| {
        let Card::Phrase(phrase_card) = card.card() else {
            return false;
        };
        !selection.excluded_card_ids.contains(id)
            && !selection.used_phrase_ids.contains(phrase_card.phrase_id())
            && phrase_tokens_all_known(phrase_card.phrase_id(), selection.known_lesson_words)
    };

    let mut due_phrases: Vec<_> = selection
        .all_cards
        .iter()
        .copied()
        .filter(|(id, card)| phrase_eligible(id, card) && card.memory().is_due())
        .collect();
    sort_phrases_by_overlap(
        &mut due_phrases,
        selection.lesson_words,
        selection.lesson_grammar_ids,
    );

    let mut new_phrases: Vec<_> = selection
        .all_cards
        .iter()
        .copied()
        .filter(|(id, card)| phrase_eligible(id, card) && card.memory().is_new())
        .collect();
    sort_phrases_by_overlap(
        &mut new_phrases,
        selection.lesson_words,
        selection.lesson_grammar_ids,
    );
    new_phrases.truncate(phrase_new_remaining);

    let mut phrase_cards = due_phrases;
    phrase_cards.extend(new_phrases);
    phrase_cards = apply_per_word_cap(phrase_cards, selection.known_lesson_words);
    phrase_cards.truncate(PHRASE_MAX_PER_LESSON);
    phrase_cards
}

/// Enforces `MAX_PHRASES_PER_WORD_IN_TAIL`: no single known lesson word may
/// anchor more than the cap tail phrases. Phrases are consumed in priority
/// order (due before new, overlap-sorted within each) so the most relevant
/// phrase wins a word's slot when contention occurs.
fn apply_per_word_cap<'a>(
    phrases: Vec<(&'a Ulid, &'a StudyCard)>,
    known_lesson_words: &HashSet<String>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    let mut word_count: HashMap<&str, usize> = HashMap::new();
    let mut kept = Vec::with_capacity(phrases.len());

    for (id, card) in phrases {
        let Card::Phrase(phrase_card) = card.card() else {
            continue;
        };
        let Some(entry) = crate::dictionary::phrase::get_index_entry(phrase_card.phrase_id())
        else {
            continue;
        };
        let over_cap = entry.tokens().iter().any(|token| {
            known_lesson_words.contains(token)
                && word_count.get(token.as_str()).copied().unwrap_or(0)
                    >= MAX_PHRASES_PER_WORD_IN_TAIL
        });
        if over_cap {
            continue;
        }
        for token in entry.tokens() {
            if known_lesson_words.contains(token) {
                *word_count.entry(token.as_str()).or_insert(0) += 1;
            }
        }
        kept.push((id, card));
    }

    kept
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

    let new_in_progress: Vec<(&Ulid, String)> = core_vocab
        .iter()
        .filter(|(id, _)| {
            knowledge_set
                .get_card(*id)
                .map(|sc| sc.memory().is_new() || sc.memory().is_in_progress())
                .unwrap_or(false)
        })
        .map(|(id, word)| (id, word.clone()))
        .collect();
    let known: Vec<(&Ulid, String)> = core_vocab
        .iter()
        .filter(|(id, _)| {
            knowledge_set
                .get_card(*id)
                .map(|sc| sc.memory().is_known_card())
                .unwrap_or(false)
        })
        .map(|(id, word)| (id, word.clone()))
        .collect();

    let mut generator = LessonViewGenerator::new(knowledge_set);

    let mut assignments = build_phrase_assignments(
        &new_in_progress,
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
/// already-mastered lesson words and share the remaining new-phrase budget with
/// the interleaved section (due phrases are free).
pub(crate) fn add_tail_phrases(
    mut lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    used_phrase_ids: &HashSet<Ulid>,
    phrase_new_budget: usize,
) -> LessonData {
    let mut all_cards = knowledge_set.study_cards().iter().collect::<Vec<_>>();
    all_cards.sort_by_key(|(_, card)| card.memory().next_review_date());

    let excluded: HashSet<Ulid> = lesson_data.cards.iter().map(|(id, _)| *id).collect();

    let lesson_study_refs: Vec<(&Ulid, &StudyCard)> = lesson_data
        .cards
        .iter()
        .filter_map(|(id, _)| knowledge_set.get_card(*id).map(|sc| (id, sc)))
        .collect();
    let (lesson_words, lesson_grammar_ids) =
        extract_lesson_content(lesson_study_refs.iter().copied());
    let known_lesson_words = extract_known_lesson_words(lesson_study_refs.iter().copied());

    let selection = TailPhraseSelection {
        all_cards: &all_cards,
        excluded_card_ids: &excluded,
        used_phrase_ids,
        known_lesson_words: &known_lesson_words,
        lesson_words: &lesson_words,
        lesson_grammar_ids: &lesson_grammar_ids,
        new_phrase_budget: phrase_new_budget,
    };
    let phrase_cards = collect_phrase_cards(&selection);

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
    use crate::domain::knowledge::{GrammarRuleCard, KanjiCard, PhraseCard, VocabularyCard};
    use crate::domain::value_objects::Question;
    use crate::domain::{RateMode, Rating};

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

        let (words, grammar_ids) = extract_lesson_content(refs.iter().copied());

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

        let (words, grammar_ids) = extract_lesson_content(refs.iter().copied());

        assert!(words.is_empty());
        assert!(grammar_ids.is_empty());
    }

    #[test]
    fn extract_lesson_content_empty_input() {
        let refs: Vec<(&Ulid, &StudyCard)> = vec![];
        let (words, grammar_ids) = extract_lesson_content(refs.iter().copied());
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

        let (words, _) = extract_lesson_content(refs.iter().copied());

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

    // --- Tail phrase selection (Slice-3) ---
    //
    // The phrase index is a process-wide `OnceLock`; only one index can live in
    // a test binary. These tests reuse the exact 4-phrase fixture also used by
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

    fn ensure_test_phrase_index() {
        PHRASE_INDEX_INIT.get_or_init(|| {
            if crate::dictionary::phrase::is_phrases_loaded() {
                return;
            }
            let index_json = r#"{"v":1,"h":"test","total":4,"phrases":[
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["test","hello"],"c":0},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["test","bye"],"c":0,"g":["01KJ9AVWBGC2BT0DMFPDYYFEWB"]},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HN","t":["test","morning"],"c":0,"g":["01KJ9AVWBGC2BT0DMFPDYYFEWB","01G00000000000000024000000"]},
                {"i":"01KPJ5S3N1DRFFD236Z4EZ03HM","t":["test","thanks"],"c":0}
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
    fn phrase_tokens_all_known_true_when_all_tokens_known() {
        ensure_test_phrase_index();

        let known = full_known_set();
        assert!(phrase_tokens_all_known(&phrase_id_hello(), &known));
        assert!(phrase_tokens_all_known(&phrase_id_bye(), &known));
    }

    #[test]
    fn phrase_tokens_all_known_false_when_token_missing() {
        ensure_test_phrase_index();

        let partial: HashSet<String> = ["test", "hello"].iter().map(|s| s.to_string()).collect();
        assert!(!phrase_tokens_all_known(&phrase_id_bye(), &partial));
    }

    #[test]
    fn phrase_tokens_all_known_false_when_index_missing_entry() {
        ensure_test_phrase_index();

        let known = full_known_set();
        assert!(!phrase_tokens_all_known(&Ulid::new(), &known));
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
    ) -> TailPhraseSelection<'a> {
        TailPhraseSelection {
            all_cards,
            excluded_card_ids: empty_ulid_set(),
            used_phrase_ids: used,
            known_lesson_words: known,
            lesson_words: known,
            lesson_grammar_ids: empty_ulid_set(),
            new_phrase_budget: 20,
        }
    }

    #[test]
    fn tail_phrases_contain_only_known_words() {
        ensure_test_phrase_index();

        let owned = make_phrase_study_cards(&[phrase_id_hello(), phrase_id_bye(), Ulid::new()]);
        let unknown_phrase = match owned[2].1.card() {
            Card::Phrase(p) => *p.phrase_id(),
            _ => unreachable!("third card is a phrase card"),
        };
        let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

        let known = full_known_set();
        let empty_used = HashSet::new();
        let selection = tail_selection(&all_cards, &known, &empty_used);
        let result = collect_phrase_cards(&selection);

        let selected = selected_phrase_ids(&result);
        assert!(
            selected.contains(&phrase_id_hello()),
            "phrase with all-known tokens should be selected"
        );
        assert!(
            selected.contains(&phrase_id_bye()),
            "phrase with all-known tokens should be selected"
        );
        assert!(
            !selected.contains(&unknown_phrase),
            "phrase whose phrase_id is absent from the index must be excluded"
        );
    }

    #[test]
    fn tail_phrases_respect_per_word_cap() {
        ensure_test_phrase_index();

        // All four phrases share the token "test"; with MAX_PHRASES_PER_WORD_IN_TAIL=2
        // only two of them may enter the tail even though every token is known.
        let owned = make_phrase_study_cards(&[
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ]);
        let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

        let known = full_known_set();
        let empty_used = HashSet::new();
        let selection = tail_selection(&all_cards, &known, &empty_used);
        let result = collect_phrase_cards(&selection);

        assert!(
            result.len() <= MAX_PHRASES_PER_WORD_IN_TAIL,
            "Tail phrases sharing a word should be capped at {MAX_PHRASES_PER_WORD_IN_TAIL}, got {}",
            result.len()
        );
        assert_eq!(
            result.len(),
            MAX_PHRASES_PER_WORD_IN_TAIL,
            "With four all-known phrases sharing one word exactly {MAX_PHRASES_PER_WORD_IN_TAIL} should be kept"
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

        let selection = tail_selection(&all_cards, &known, &used);
        let result = collect_phrase_cards(&selection);

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
        // mutually exclusive with the 4-phrase fixture used across the lib test
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

    /// Interleaving targets new/in-progress vocab and deliberately skips
    /// high-difficulty vocab: phrases anchored to the high-difficulty word must
    /// not appear, while the new and in-progress words receive their phrases.
    #[test]
    fn interleaved_phrases_target_new_and_in_progress() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
        let morning_sc = ks
            .create_card(vocab_card("morning"))
            .expect("create morning");
        let bye_sc = ks.create_card(vocab_card("bye")).expect("create bye");
        for pid in [phrase_id_hello(), phrase_id_morning(), phrase_id_bye()] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        // in_progress: rated Good once, stability stays under the known threshold.
        ks.rate_card(*hello_sc.card_id(), Rating::Good, RateMode::StandardLesson)
            .expect("rate hello");
        // High difficulty: repeated Again under ShortTerm drives difficulty up.
        for _ in 0..3 {
            ks.rate_card(*bye_sc.card_id(), Rating::Again, RateMode::ShortTerm)
                .expect("rate bye");
        }
        // morning stays new (untouched).

        let hello_id = *hello_sc.card_id();
        let morning_id = *morning_sc.card_id();
        let bye_id = *bye_sc.card_id();
        assert!(
            ks.get_card(morning_id).unwrap().memory().is_new(),
            "fixture sanity: morning should be new"
        );
        assert!(
            ks.get_card(bye_id).unwrap().memory().is_high_difficulty(),
            "fixture sanity: bye should be high difficulty"
        );
        assert!(
            ks.get_card(hello_id).unwrap().memory().is_in_progress(),
            "fixture sanity: hello should be in progress"
        );

        let lesson = LessonData {
            cards: vec![
                (hello_id, lesson_card_for(vocab_card("hello"))),
                (morning_id, lesson_card_for(vocab_card("morning"))),
                (bye_id, lesson_card_for(vocab_card("bye"))),
            ],
            core_count: 3,
        };

        let mut used = HashSet::new();
        let mut budget = 5;
        let result = add_interleaved_phrases(lesson, &ks, &mut used, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.contains(&phrase_id_hello()),
            "new/in-progress anchor 'hello' should receive its phrase"
        );
        assert!(
            phrases.contains(&phrase_id_morning()),
            "new anchor 'morning' should receive its phrase"
        );
        assert!(
            !phrases.contains(&phrase_id_bye()),
            "high-difficulty anchor 'bye' must not receive an interleaved phrase"
        );
    }

    /// With no new/in-progress vocab, interleaving falls back to known vocab.
    /// High-difficulty vocab is never used as an anchor, even in the fallback.
    #[test]
    fn interleaved_phrases_fallback_to_known() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
        let bye_sc = ks.create_card(vocab_card("bye")).expect("create bye");
        for pid in [phrase_id_hello(), phrase_id_bye()] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        ks.mark_card_as_known(*hello_sc.card_id())
            .expect("mark hello known");
        for _ in 0..3 {
            ks.rate_card(*bye_sc.card_id(), Rating::Again, RateMode::ShortTerm)
                .expect("rate bye");
        }

        let hello_id = *hello_sc.card_id();
        let bye_id = *bye_sc.card_id();
        assert!(ks.get_card(hello_id).unwrap().memory().is_known_card());
        assert!(ks.get_card(bye_id).unwrap().memory().is_high_difficulty());

        let lesson = LessonData {
            cards: vec![
                (hello_id, lesson_card_for(vocab_card("hello"))),
                (bye_id, lesson_card_for(vocab_card("bye"))),
            ],
            core_count: 2,
        };

        let mut used = HashSet::new();
        let mut budget = 5;
        let result = add_interleaved_phrases(lesson, &ks, &mut used, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.contains(&phrase_id_hello()),
            "fallback should anchor a phrase to the known word"
        );
        assert!(
            !phrases.contains(&phrase_id_bye()),
            "high-difficulty word must not anchor a phrase even via fallback"
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
