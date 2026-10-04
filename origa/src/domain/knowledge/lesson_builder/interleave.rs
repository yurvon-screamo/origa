use super::*;

/// How many phrases may be interleaved next to a single anchor word inside the
/// core section. Two cards let the learner meet the word in context shortly
/// after the standalone review without saturating the lesson.
pub(super) const INTERLEAVED_PHRASES_PER_WORD: usize = 2;

/// Reorders the core section (`cards[..core_count]`) so kanji and grammar are
/// spread across the lesson instead of clustering at the end. Vocab acts as the
/// separator spine: kanji and grammar are dealt round-robin into the `V+1` gaps
/// between vocab cards, bounding the longest same-type run to
/// `⌈count/(V+1)⌉`. The bound depends only on card counts, not on the shuffled
/// within-type order, so it is deterministic. When the core has no vocab there
/// is no separator to spread with, so the layout is left untouched.
pub(crate) fn interleave_core_by_type(mut lesson_data: LessonData) -> LessonData {
    let core_count = lesson_data.core_count;
    if core_count <= 1 {
        return lesson_data;
    }

    let vocab_count = lesson_data.cards[..core_count]
        .iter()
        .filter(|(_, lc)| CardType::from(lc.card()) == CardType::Vocabulary)
        .count();
    if vocab_count == 0 {
        return lesson_data;
    }

    let (mut vocab, mut kanji, mut grammar, mut counter, mut other) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for card in lesson_data.cards.drain(..core_count) {
        match CardType::from(card.1.card()) {
            CardType::Vocabulary => vocab.push(card),
            CardType::Kanji => kanji.push(card),
            CardType::Grammar => grammar.push(card),
            // Собственная очередь: счётные суффиксы раздаются round-robin
            // в vocab-промежутки, как кандзи и грамматика (issue #415).
            CardType::Counter => counter.push(card),
            CardType::Phrase => other.push(card),
        }
    }

    let num_gaps = vocab_count + 1;
    let mut gap_kanji: Vec<Vec<(Ulid, LessonCard)>> = (0..num_gaps).map(|_| Vec::new()).collect();
    for (i, card) in kanji.into_iter().enumerate() {
        gap_kanji[i % num_gaps].push(card);
    }
    let mut gap_grammar: Vec<Vec<(Ulid, LessonCard)>> = (0..num_gaps).map(|_| Vec::new()).collect();
    for (i, card) in grammar.into_iter().enumerate() {
        gap_grammar[i % num_gaps].push(card);
    }
    let mut gap_counter: Vec<Vec<(Ulid, LessonCard)>> = (0..num_gaps).map(|_| Vec::new()).collect();
    for (i, card) in counter.into_iter().enumerate() {
        gap_counter[i % num_gaps].push(card);
    }

    let mut new_core: Vec<(Ulid, LessonCard)> = Vec::with_capacity(core_count);
    new_core.append(&mut gap_kanji[0]);
    new_core.append(&mut gap_grammar[0]);
    new_core.append(&mut gap_counter[0]);
    for (i, vcard) in vocab.into_iter().enumerate() {
        new_core.push(vcard);
        new_core.append(&mut gap_kanji[i + 1]);
        new_core.append(&mut gap_grammar[i + 1]);
        new_core.append(&mut gap_counter[i + 1]);
    }
    new_core.append(&mut other);

    lesson_data.cards.splice(..0, new_core);
    lesson_data
}

/// Interleaves phrase cards into the core stream so each phrase appears after
/// its anchor word with at least `gap` other cards between them. The invariant
/// `phrase_position > word_position` is preserved even when the lesson is too
/// short to honour the gap (remaining phrases flush at the end).
pub(super) fn interleave_with_gap(
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
pub(super) fn collect_interleaved_phrases_for_word<'a>(
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
pub(super) struct InterleavePicker<'a> {
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
        entries: &[crate::dictionary::phrase::IndexEntry],
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

/// Collects up to `INTERLEAVED_PHRASES_PER_WORD` phrase card ids per anchor
/// word. Due phrases win slots for free; new phrases consume the shared budget.
/// Dedupes by word text so the same word is never processed twice. Returns
/// owned card ids (not references) so the shared `&mut` borrow of the budget
/// set is released between iterations.
pub(super) fn collect_anchored_phrase_card_ids(
    targets: &[(Ulid, String)],
    phrase_cards_by_id: &HashMap<Ulid, (&Ulid, &StudyCard)>,
    in_lesson: &HashSet<Ulid>,
    used_phrase_ids: &mut HashSet<Ulid>,
    phrase_new_budget: &mut usize,
) -> Vec<Ulid> {
    let mut seen_words: HashSet<&str> = HashSet::new();
    let mut out = Vec::new();
    for (_, word_text) in targets {
        if !seen_words.insert(word_text.as_str()) {
            continue;
        }
        let picked = collect_interleaved_phrases_for_word(
            word_text,
            phrase_cards_by_id,
            in_lesson,
            used_phrase_ids,
            phrase_new_budget,
        );
        for (card_id, _) in picked {
            out.push(card_id);
        }
    }
    out
}
