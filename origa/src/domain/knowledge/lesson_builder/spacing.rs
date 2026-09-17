use super::expansion::MIN_REPEAT_SPACING;
use super::phrases::{INTERLEAVING_GAP, place_phrases_constraint_aware};
use super::*;

/// Final layout pass: reorders the core section so consecutive showings of the
/// same `card_id` are maximally separated, removing the back-to-back
/// clustering that `expand_repeated_views` leaves when a multi-show anchor
/// sits near the end of a short core. Content (vocab/kanji/grammar, including
/// multi-show copies) is re-dealt via `deal_by_card_id`; phrases are then
/// re-placed through the existing constraint-aware placer so the
/// phrase-after-word invariant (PR #203) is re-derived from the new content
/// order. `core_count` and the tail section are preserved: the dealt core has
/// the same length, and inserting phrases between content cards only widens
/// the gaps between them.
pub(crate) fn redistribute_core_for_spacing(mut lesson_data: LessonData) -> LessonData {
    let core_count = lesson_data.core_count;
    if core_count <= 1 {
        return lesson_data;
    }

    let core: Vec<(Ulid, LessonCard)> = lesson_data.cards.drain(..core_count).collect();
    let (content, phrases): (Vec<_>, Vec<_>) = core
        .into_iter()
        .partition(|(_, lc)| !matches!(lc.card(), Card::Phrase(_)));

    let core_vocab: Vec<(Ulid, String)> = content
        .iter()
        .filter_map(|(id, lc)| match lc.card() {
            Card::Vocabulary(v) => Some((*id, v.word().text().to_string())),
            _ => None,
        })
        .collect();

    let dealt = deal_by_card_id(content);
    let reordered = if phrases.is_empty() {
        dealt
    } else if dealt.is_empty() {
        // Defensive: a core made only of phrases has no content to anchor or
        // reorder against — keep the phrases in place rather than dropping them
        // (unreachable in the real pipeline, where the core always holds at
        // least one non-phrase card).
        phrases
    } else {
        place_phrases_constraint_aware(dealt, phrases, &core_vocab, INTERLEAVING_GAP)
    };

    lesson_data.cards.splice(..0, reordered);
    lesson_data
}

/// Reorders `content` so consecutive showings of the same `card_id` stay at
/// least `MIN_REPEAT_SPACING` apart whenever the slot count allows it, using
/// the Task-Scheduler greedy: at each output position the slot of the
/// available group with the highest remaining count is emitted ("available" =
/// the group's previous showing is more than `MIN_REPEAT_SPACING` positions
/// back). This is the canonical min-distance construction — it spreads a LONE
/// multi-show card across the whole core (its copies land exactly
/// `MIN_REPEAT_SPACING + 1` apart when it is the bottleneck) while still
/// interleaving many multi-show cards. When the core is structurally too small
/// to honour every gap (the counts cannot fit), the greedy degrades to
/// best-effort by emitting the most-loaded group anyway — the only case where
/// `MIN_REPEAT_SPACING` may be violated, matching the upstream best-effort
/// contract. Within-group view order (`[primary, copy1, copy2]`) and the
/// "primary is the first showing" invariant are preserved: a group's queue is
/// always drained front-to-back.
pub(super) fn deal_by_card_id(content: Vec<(Ulid, LessonCard)>) -> Vec<(Ulid, LessonCard)> {
    let n = content.len();
    if n <= 1 {
        return content;
    }

    // (queue of slots in original view order, first_original_index)
    let mut queues: Vec<(VecDeque<(Ulid, LessonCard)>, usize)> = Vec::new();
    let mut index_by_card: HashMap<Ulid, usize> = HashMap::new();
    for (i, slot) in content.into_iter().enumerate() {
        let card_id = slot.1.card_id();
        match index_by_card.get(&card_id) {
            Some(&qi) => queues[qi].0.push_back(slot),
            None => {
                index_by_card.insert(card_id, queues.len());
                queues.push((VecDeque::from([slot]), i));
            },
        }
    }

    let m = queues.len();
    let mut last_pos: Vec<Option<usize>> = vec![None; m];
    let mut result: Vec<(Ulid, LessonCard)> = Vec::with_capacity(n);

    for p in 0..n {
        // Spacing-respecting pass first; fall back to the most-loaded group
        // (ignoring cooldown) only when nothing is available — the sole path
        // that can violate MIN_REPEAT_SPACING, reachable on a structurally
        // overloaded core.
        let chosen = pick_group(&queues, &last_pos, p, false)
            .or_else(|| pick_group(&queues, &last_pos, p, true));
        let Some(qi) = chosen else {
            break;
        };
        if let Some(slot) = queues[qi].0.pop_front() {
            last_pos[qi] = Some(p);
            result.push(slot);
        }
    }
    result
}

/// Selects the next group to emit at output position `p`. With `force` false
/// only groups whose previous showing is more than `MIN_REPEAT_SPACING`
/// positions back qualify (the spacing-respecting pass); with `force` true the
/// cooldown is ignored (the best-effort fallback). The highest remaining count
/// wins; ties are broken by earliest first-occurrence so the pick is
/// deterministic regardless of the random slot/card ULIDs.
pub(super) fn pick_group(
    queues: &[(VecDeque<(Ulid, LessonCard)>, usize)],
    last_pos: &[Option<usize>],
    p: usize,
    force: bool,
) -> Option<usize> {
    let mut best: Option<(usize, usize, usize)> = None;
    for (qi, (queue, first_idx)) in queues.iter().enumerate() {
        let rem = queue.len();
        if rem == 0 {
            continue;
        }
        let cooled = last_pos[qi].map_or(true, |lp| p - lp > MIN_REPEAT_SPACING);
        if !force && !cooled {
            continue;
        }
        match best {
            None => best = Some((qi, rem, *first_idx)),
            Some((_, b_rem, b_first)) => {
                if rem > b_rem || (rem == b_rem && *first_idx < b_first) {
                    best = Some((qi, rem, *first_idx));
                }
            },
        }
    }
    best.map(|(qi, _, _)| qi)
}
