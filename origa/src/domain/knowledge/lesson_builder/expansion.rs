use super::*;

/// Minimum number of cards that must sit between two consecutive showings
/// of the same primary card after multi-show expansion. Prevents
/// back-to-back ratings of the same underlying StudyCard.
pub(super) const MIN_REPEAT_SPACING: usize = 3;

/// Per-card target number of showings, derived from the FSRS memory state.
/// Only high-difficulty cards get repeated showings (2); new, in-progress, and
/// known cards keep their original single showing.
pub(super) fn target_showings(study_card: &StudyCard) -> usize {
    let memory = study_card.memory();
    if memory.is_high_difficulty() { 2 } else { 1 }
}

/// Decides whether a primary card slot should be expanded into multiple
/// showings, and if so returns the COPY views (the extra showings beyond the
/// primary). The primary itself is the probabilistic view `apply_view` already
/// assigned upstream (`primary_view`); copies are drawn from
/// `candidate_views_for_repeat` with the primary's variant removed, so every
/// showing of the card uses a distinct `LessonCardView` variant. Returns an
/// empty vector when the card is exempt (not primary, not a multi-show type,
/// target is 1, or no distinct copy variant is available).
pub(super) fn compute_expansion_views(
    generator: &mut LessonViewGenerator,
    knowledge_set: &KnowledgeSet,
    primary_card_ids: &HashSet<Ulid>,
    card_id: Ulid,
    card_type: CardType,
    primary_view: &LessonCardView,
) -> Vec<LessonCardView> {
    if !primary_card_ids.contains(&card_id) {
        return Vec::new();
    }
    if !matches!(
        card_type,
        CardType::Vocabulary | CardType::Kanji | CardType::Grammar
    ) {
        return Vec::new();
    }

    let Some(study_card) = knowledge_set.get_card(card_id) else {
        return Vec::new();
    };

    let target = target_showings(study_card);
    if target <= 1 {
        return Vec::new();
    }

    let primary_disc = std::mem::discriminant(primary_view);
    let mut candidates =
        generator.candidate_views_for_repeat(study_card, study_card.is_new(), &mut rand::rng());
    candidates.retain(|view| std::mem::discriminant(view) != primary_disc);
    candidates.truncate(target.saturating_sub(1));

    candidates
}

/// Multiplies primary (non-phrase) cards across multiple distinct views when
/// their FSRS state demands it. Each copy occupies its own slot (unique slot
/// id) but shares the underlying StudyCard id (`card_id`), so every showing is
/// rated independently. Companion cards, phrases, and primary cards with a
/// single-show target (or whose guards clamp to a single distinct view) keep
/// their original slot unchanged.
///
/// Runs last in the pipeline so every upstream step still operates by slot id.
/// `core_count` grows by the number of copies added so the tail-vs-core
/// boundary stays contiguous.
pub(crate) fn expand_repeated_views(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    native_language: NativeLanguage,
    primary_card_ids: &HashSet<Ulid>,
) -> LessonData {
    let original_cards = lesson_data.cards;
    let core_count_before = lesson_data.core_count;

    let (core_cards, tail_cards) = original_cards.split_at(core_count_before);

    let mut generator = LessonViewGenerator::new(knowledge_set, native_language);
    let mut new_core: Vec<(Ulid, LessonCard)> = Vec::with_capacity(core_cards.len() * 2);
    let mut pending: Vec<(usize, Ulid, LessonCardView, bool)> = Vec::new();

    for (slot_id, lc) in core_cards.iter() {
        let card_id = lc.card_id();
        let card_type = CardType::from(lc.card());
        let is_short_term = lc.is_short_term();
        let primary_view = lc.view().clone();

        let copy_views = compute_expansion_views(
            &mut generator,
            knowledge_set,
            primary_card_ids,
            card_id,
            card_type,
            &primary_view,
        );

        if copy_views.is_empty() {
            new_core.push((*slot_id, lc.clone()));
            drain_pending(&mut new_core, &mut pending);
            continue;
        }

        new_core.push((
            *slot_id,
            LessonCard::new(card_id, primary_view, is_short_term),
        ));

        // Each extra showing of the same card_id must land MIN_REPEAT_SPACING
        // cards after the PREVIOUS showing of that card_id. Translating
        // "N cards between" into index deltas: positions differ by N+1. The
        // anchor sits at index `new_core.len() - 1`, so the first extra view
        // targets `new_core.len() + MIN_REPEAT_SPACING` (== anchor_idx +
        // MIN_REPEAT_SPACING + 1). Subsequent views step by the same delta so
        // every gap honours the same invariant.
        let mut next_min_pos = new_core.len() + MIN_REPEAT_SPACING;
        for view in copy_views {
            pending.push((next_min_pos, card_id, view, is_short_term));
            next_min_pos += MIN_REPEAT_SPACING + 1;
        }

        drain_pending(&mut new_core, &mut pending);
    }

    new_core = distribute_pending_with_spacing(new_core, pending);
    let added = new_core.len() - core_cards.len();

    let mut final_cards = new_core;
    final_cards.extend(tail_cards.iter().cloned());

    LessonData {
        cards: final_cards,
        core_count: core_count_before + added,
    }
}

/// Flushes any pending expansion copies whose minimum position has been
/// reached. Remaining copies stay in `pending` for a future iteration.
pub(super) fn drain_pending(
    new_core: &mut Vec<(Ulid, LessonCard)>,
    pending: &mut Vec<(usize, Ulid, LessonCardView, bool)>,
) {
    let mut deferred = Vec::new();
    for (min_pos, p_card_id, p_view, p_short) in pending.drain(..) {
        if new_core.len() >= min_pos {
            new_core.push((Ulid::new(), LessonCard::new(p_card_id, p_view, p_short)));
        } else {
            deferred.push((min_pos, p_card_id, p_view, p_short));
        }
    }
    *pending = deferred;
}

/// Distributes expansion copies that did not fit during the main loop.
/// Each copy is inserted at the earliest position that keeps at least
/// `MIN_REPEAT_SPACING` cards between it and the previous showing of the
/// same `card_id`, instead of blindly appending the leftovers back-to-back
/// (which would make consecutive showings of one card land adjacent and
/// defeat the purpose of the spacing rule).
///
/// Contract: spacing is guaranteed only when the assembled core is large
/// enough to absorb every pending copy at its required gap. On a
/// too-short lesson — anchor near the end of the core, or a single-card
/// core whose target forces more copies than the buffer can hold — the
/// target index is clamped to `core.len()` and copies cluster at the end
/// of the core section. This is the only mathematically unreachable case
/// (the main loop already spaces copies whenever the core has room, see
/// `expand_enforces_min_spacing_between_consecutive_showings`). The
/// "past `core_count` is phrase-only" invariant is preserved because
/// copies are inserted strictly inside the core section; tail phrases
/// remain at the very end of the lesson.
pub(super) fn distribute_pending_with_spacing(
    mut core: Vec<(Ulid, LessonCard)>,
    mut pending: Vec<(usize, Ulid, LessonCardView, bool)>,
) -> Vec<(Ulid, LessonCard)> {
    if pending.is_empty() {
        return core;
    }
    pending.sort_by_key(|(min_pos, _, _, _)| *min_pos);

    for (min_pos, card_id, view, is_short_term) in pending {
        let last_pos = core
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (_, lc))| lc.card_id() == card_id)
            .map(|(idx, _)| idx);
        let spacing_target = last_pos
            .map(|prev| prev + MIN_REPEAT_SPACING + 1)
            .unwrap_or(min_pos);
        let target = spacing_target.max(min_pos).min(core.len());
        core.insert(
            target,
            (Ulid::new(), LessonCard::new(card_id, view, is_short_term)),
        );
    }
    core
}
