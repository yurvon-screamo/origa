use super::interleave::{collect_anchored_phrase_card_ids, interleave_with_gap};
use super::*;

/// Per-lesson cap on no-anchor (formerly "tail") phrases — phrases whose
/// tokens are all known but none of which anchor to a lesson vocab word. They
/// are MIXED INTO the lesson (no dedicated end zone, see PR #203). This slot
/// is independent of the new-anchored-phrase allowance: up to this many
/// no-anchor phrases (due + new) appear every lesson.
pub(super) const TAIL_PHRASE_PER_LESSON: usize = 5;

/// Tail phrases only reinforce already-mastered material: no single known word
/// may appear in more than this many tail phrases, otherwise a frequent word
/// (e.g. する) would crowd out the entire tail.
pub(crate) const MAX_PHRASES_PER_WORD_IN_TAIL: usize = 1;

/// Minimum number of other cards that must sit between an anchor word and its
/// first interleaved phrase, so the phrase does not leak the answer back into
/// the word rating. Degrades to "as late as possible" on short lessons.
pub(super) const INTERLEAVING_GAP: usize = 2;

/// A phrase is tail-eligible when every non-particle token it references is
/// part of the known vocabulary pool. Grammatical particles (は, が, を, …)
/// attach to words rather than carrying standalone meaning, so they never
/// block eligibility. The pool spans the ENTIRE knowledge_set (built by
/// `collect_known_vocabulary_words`), not just lesson cards.
pub(super) fn phrase_tail_eligible(phrase_id: &Ulid, known_pool: &HashSet<String>) -> bool {
    let Some(entry) = crate::dictionary::phrase::get_index_entry(phrase_id) else {
        return false;
    };
    entry.tokens().iter().all(|token| {
        crate::domain::grammar::is_grammatical_particle(token) || known_pool.contains(token)
    })
}

/// Bundles the immutable inputs driving no-anchor ("tail") phrase selection.
/// Grouping them keeps `collect_phrase_cards` below the argument-count
/// threshold and makes the selection context explicit at the call site. The
/// no-anchor section does NOT draw from `phrase_new_budget` (that budget
/// governs only anchored phrases), so it is intentionally absent here — see
/// `TAIL_PHRASE_PER_LESSON` for the per-lesson cap.
pub(super) struct TailPhraseSelection<'a> {
    pub(super) all_cards: &'a [(&'a Ulid, &'a StudyCard)],
    pub(super) excluded_card_ids: &'a HashSet<Ulid>,
    pub(super) used_phrase_ids: &'a HashSet<Ulid>,
    pub(super) known_pool: &'a HashSet<String>,
}

pub(super) fn collect_phrase_cards<'a>(
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
    // `add_phrases`), so filtering preserves the scheduling order without
    // an explicit secondary sort.
    let mut cap = PerWordCap::new(selection.known_pool);
    let mut phrase_cards: Vec<(&'a Ulid, &'a StudyCard)> = Vec::new();

    // Due no-anchor phrases first — free of any budget cost, but they still
    // occupy per-word cap slots so a frequent word cannot crowd out the
    // section through scheduling pressure alone.
    for (id, card) in selection.all_cards.iter().copied() {
        if !phrase_eligible(&id, &card) || !card.memory().is_due() {
            continue;
        }
        if cap.try_admit(card) {
            phrase_cards.push((id, card));
        }
    }

    // New no-anchor phrases are admitted on the per-word cap alone: they do
    // NOT decrement `phrase_new_budget` (reserved for anchored phrases), so
    // a depleted budget never starves the no-anchor section. The total is
    // bounded per lesson by `TAIL_PHRASE_PER_LESSON` via the truncate below.
    for (id, card) in selection.all_cards.iter().copied() {
        if !phrase_eligible(&id, &card) || !card.memory().is_new() {
            continue;
        }
        if cap.try_admit(card) {
            phrase_cards.push((id, card));
        }
    }

    phrase_cards.truncate(TAIL_PHRASE_PER_LESSON);
    phrase_cards
}

/// Streaming enforcer of `MAX_PHRASES_PER_WORD_IN_TAIL`. `try_admit` returns
/// `true` and reserves the phrase's known-word slots when the phrase still
/// fits the cap, `false` when at least one anchored word is already saturated.
/// Phrases are consumed in admission order (due before new) so the most
/// relevant phrase wins a word's slot when contention occurs.
pub(super) struct PerWordCap<'a> {
    known_pool: &'a HashSet<String>,
    word_count: HashMap<String, usize>,
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
                *self.word_count.entry(token.clone()).or_insert(0) += 1;
            }
        }
        true
    }
}

/// Inserts phrases (anchored + no-anchor) into the core so each phrase lands
/// after the first showing of every lesson-vocab word it references,
/// distributing the rest instead of dumping them at the end. All phrases become
/// part of the core (`core_count` grows to the whole lesson), removing the
/// dedicated tail zone.
///
/// `phrase_new_budget` is the PER-LESSON allowance of NEW ANCHORED phrases,
/// initialized by the caller from
/// [`crate::domain::DailyBudget::new_phrases_per_lesson`] — fresh for every
/// lesson of the day, deliberately not a daily budget (an evening lesson must
/// not be starved by a morning one). It bounds new anchored phrases only
/// (per-word cap `INTERLEAVED_PHRASES_PER_WORD` also applies; there is
/// deliberately no per-lesson TOTAL cap on anchored phrases). NEW no-anchor
/// phrases are admitted independently, capped per lesson by
/// `TAIL_PHRASE_PER_LESSON`, so a depleted budget never starves the
/// no-anchor section.
pub(crate) fn add_phrases(
    mut lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    native_language: NativeLanguage,
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
    let mut used_phrase_ids: HashSet<Ulid> = HashSet::new();

    let core_vocab: Vec<(Ulid, String)> = lesson_data.cards[..core_count]
        .iter()
        .filter_map(|(id, lc)| match lc.card() {
            Card::Vocabulary(v) => Some((*id, v.word().text().to_string())),
            _ => None,
        })
        .collect();

    // Anchored interleaving reinforces vocab still being learned; fall back to
    // known vocab only when no non-known anchor yields any phrase.
    let (non_known, known) = core_vocab
        .iter()
        .cloned()
        .partition::<Vec<_>, _>(|(id, _)| {
            knowledge_set
                .get_card(*id)
                .map(|sc| !sc.memory().is_known_card())
                .unwrap_or(false)
        });

    let mut selected_ids: Vec<Ulid> = collect_anchored_phrase_card_ids(
        &non_known,
        &phrase_cards_by_id,
        &in_lesson,
        &mut used_phrase_ids,
        phrase_new_budget,
    );
    if selected_ids.is_empty() && !known.is_empty() {
        selected_ids = collect_anchored_phrase_card_ids(
            &known,
            &phrase_cards_by_id,
            &in_lesson,
            &mut used_phrase_ids,
            phrase_new_budget,
        );
    }

    // No-anchor phrases: whole known-pool eligibility, per-word cap, and a
    // per-lesson count bounded by `TAIL_PHRASE_PER_LESSON`. They do NOT draw
    // from `phrase_new_budget` (which stays reserved for the anchored pass
    // above), so they survive even a depleted budget.
    let mut all_cards = knowledge_set.study_cards().iter().collect::<Vec<_>>();
    all_cards.sort_by_key(|(_, card)| card.memory().next_review_date());
    let known_pool = crate::domain::knowledge::collect_known_vocabulary_words(
        knowledge_set.study_cards().values(),
        true,
    );
    let mut tail_selection = TailPhraseSelection {
        all_cards: &all_cards,
        excluded_card_ids: &in_lesson,
        used_phrase_ids: &used_phrase_ids,
        known_pool: &known_pool,
    };
    let tail_cards = collect_phrase_cards(&mut tail_selection);
    selected_ids.extend(tail_cards.iter().map(|(id, _)| **id));

    if selected_ids.is_empty() {
        return lesson_data;
    }

    let mut generator = LessonViewGenerator::new(knowledge_set, native_language);
    let phrase_lessons: Vec<(Ulid, LessonCard)> = selected_ids
        .iter()
        .filter_map(|card_id| {
            let sc = knowledge_set.get_card(*card_id)?;
            let view = generator.apply_view(sc, sc.is_new(), &mut rand::rng());
            Some((*card_id, LessonCard::new(*card_id, view, false)))
        })
        .collect();

    let core_cards = std::mem::take(&mut lesson_data.cards);
    lesson_data.cards =
        place_phrases_constraint_aware(core_cards, phrase_lessons, &core_vocab, INTERLEAVING_GAP);
    lesson_data.core_count = lesson_data.cards.len();
    lesson_data
}

/// Places `phrases` among `core_cards` honouring the contamination constraint:
/// a phrase that references a lesson-vocab word must appear AFTER the first
/// showing of every such word. Each phrase is released at the latest
/// first-occurrence among its anchor words (so it follows all of them), then
/// handed to `interleave_with_gap` which keeps it `INTERLEAVING_GAP` cards past
/// the release point. Phrases with no anchor in the lesson are distributed at
/// even intervals so they no longer pile up at the end.
pub(super) fn place_phrases_constraint_aware(
    core_cards: Vec<(Ulid, LessonCard)>,
    phrases: Vec<(Ulid, LessonCard)>,
    core_vocab: &[(Ulid, String)],
    gap: usize,
) -> Vec<(Ulid, LessonCard)> {
    let n = core_cards.len();

    let mut first_pos: HashMap<Ulid, usize> = HashMap::with_capacity(n);
    for (i, (id, _)) in core_cards.iter().enumerate() {
        first_pos.entry(*id).or_insert(i);
    }

    let word_to_card: HashMap<String, Ulid> = core_vocab
        .iter()
        .map(|(id, word)| (word.clone(), *id))
        .collect();

    let mut releases: Vec<(usize, (Ulid, LessonCard))> = Vec::with_capacity(phrases.len());
    let mut no_anchor: Vec<(Ulid, LessonCard)> = Vec::new();

    for phrase in phrases {
        let phrase_id = match phrase.1.card() {
            Card::Phrase(p) => *p.phrase_id(),
            _ => {
                no_anchor.push(phrase);
                continue;
            },
        };
        let tokens: Vec<String> = crate::dictionary::phrase::get_index_entry(&phrase_id)
            .map(|e| e.tokens().to_vec())
            .unwrap_or_default();
        let anchors: Vec<usize> = tokens
            .iter()
            .filter_map(|t| word_to_card.get(t.as_str()).copied())
            .filter_map(|card_id| first_pos.get(&card_id).copied())
            .collect();
        if anchors.is_empty() {
            no_anchor.push(phrase);
        } else {
            let max_pos = *anchors.iter().max().expect("anchors non-empty");
            releases.push((max_pos, phrase));
        }
    }

    let no_anchor_count = no_anchor.len();
    for (i, phrase) in no_anchor.into_iter().enumerate() {
        let target_idx = if n == 0 {
            0
        } else {
            (((i + 1) * n) / (no_anchor_count + 1)).min(n - 1)
        };
        releases.push((target_idx, phrase));
    }

    let mut assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>> = HashMap::new();
    for (idx, phrase) in releases {
        if let Some((release_id, _)) = core_cards.get(idx) {
            assignments.entry(*release_id).or_default().push(phrase);
        }
    }

    interleave_with_gap(core_cards, assignments, gap)
}
