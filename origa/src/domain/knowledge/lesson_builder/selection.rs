use super::*;

/// Collects the lesson core (favorites, ghost hand, due/new/known cards,
/// padding) WITHOUT any phrases. The core is shuffled here so downstream
/// interleaving sees a stable order. Phrases are attached later by
/// `add_phrases`.
///
/// Priorities within the `MAX_LESSON_SIZE` budget: favorites (pinned) →
/// ghost hand (active добивания with an open window, most overdue first,
/// no separate daily cap — «старое прежде нового») → high-difficulty due →
/// new cards (daily limit) → due in-progress/known → high-difficulty
/// padding. A card with an active добивание enters ONLY through the hand
/// (or as a favorite): core collectors and padding exclude it.
pub(crate) fn build_lesson_core(
    knowledge_set: &KnowledgeSet,
    daily_new_limit: usize,
    jlpt_content: &JlptContent,
    native_language: NativeLanguage,
    policy: NewCardPolicy,
) -> LessonData {
    let now = Utc::now();
    let mut all_cards = knowledge_set.study_cards().iter().collect::<Vec<_>>();
    all_cards.sort_by_key(|(_, card)| card.memory().next_review_date());

    let favorite_cards: Vec<_> = all_cards
        .iter()
        .filter(|(_, card)| {
            card.is_favorite()
                && !excluded_by_new_card_policy(card.card().into(), card.memory().is_new(), policy)
        })
        .copied()
        .collect();

    let favorite_ids: HashSet<Ulid> = favorite_cards.iter().map(|(id, _)| **id).collect();

    // Рука добиваний — первый резидент ядра: следующие сборщики считают
    // её в бюджете (selected_cards.len()) и не добирают её карты
    // повторно (is_core_candidate исключает активные добивания).
    let mut selected_cards = collect_ghost_hand(&all_cards, &favorite_ids, now);

    collect_core_high_difficulty(&all_cards, &mut selected_cards, &favorite_ids, now);
    let mut rng = rand::rng();
    if policy == NewCardPolicy::Inject {
        collect_core_new_cards(
            &all_cards,
            &mut selected_cards,
            &favorite_ids,
            daily_new_limit.saturating_sub(knowledge_set.new_cards_studied_today()),
            jlpt_content,
            &mut rng,
            now,
        );
    }
    fill_core_due_known(&all_cards, &mut selected_cards, &favorite_ids, now);

    let all_selected_ids = build_selected_ids(&selected_cards, &favorite_cards);
    let padding_cards = collect_padding(&all_cards, &all_selected_ids, now);

    let mut cards = build_core_lesson_cards(
        &favorite_cards,
        &selected_cards,
        &padding_cards,
        knowledge_set,
        native_language,
    );
    cards.shuffle(&mut rand::rng());
    let core_count = cards.len();

    LessonData { cards, core_count }
}

/// Core-section eligibility predicate shared by the high-difficulty, new and
/// due-known collectors: a card is a core candidate when it is neither a
/// user favorite (already pinned), nor a phrase (phrases enter the lesson
/// via the interleaved/tail pipelines, not the core), nor a card with an
/// active добивание (its only channel is the ghost hand; a favorite stays
/// pinned — the showing counts toward the ladder when the window is open).
pub(super) fn is_core_candidate(
    id: &Ulid,
    card: &StudyCard,
    favorite_ids: &HashSet<Ulid>,
    now: DateTime<Utc>,
) -> bool {
    !favorite_ids.contains(id)
        && !matches!(card.card(), Card::Phrase(_))
        && !card.memory().has_active_ghost(now)
}

/// Рука добиваний [GhostHand]: карты с активным добиванием и открытым
/// окном, наиболее просроченные первыми. Без отдельного дневного лимита —
/// ограничитель общий бюджет урока (`MAX_LESSON_SIZE` минус избранное);
/// переполнение переносится на следующий урок без потери состояния.
pub(super) fn collect_ghost_hand<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    favorite_ids: &HashSet<Ulid>,
    now: DateTime<Utc>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    let limit = MAX_LESSON_SIZE.saturating_sub(favorite_ids.len());
    let mut hand: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| !favorite_ids.contains(id) && card.memory().ghost_hand_ready(now))
        .copied()
        .collect();
    hand.sort_by_key(|(_, card)| card.memory().ghost().and_then(|g| g.due_at().copied()));
    hand.into_iter().take(limit).collect()
}

pub(super) fn collect_core_high_difficulty<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    selected_cards: &mut Vec<(&'a Ulid, &'a StudyCard)>,
    favorite_ids: &HashSet<Ulid>,
    now: DateTime<Utc>,
) {
    let limit = MAX_LESSON_SIZE.saturating_sub(selected_cards.len() + favorite_ids.len());
    if limit == 0 {
        return;
    }
    selected_cards.extend(
        all_cards
            .iter()
            .filter(|(id, card)| {
                is_core_candidate(id, card, favorite_ids, now)
                    && card.memory().is_due()
                    && card.memory().is_high_difficulty()
            })
            .take(limit)
            .copied(),
    );
}

pub(super) fn collect_core_new_cards<'a, R: rand::Rng>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    selected_cards: &mut Vec<(&'a Ulid, &'a StudyCard)>,
    favorite_ids: &HashSet<Ulid>,
    daily_new_remaining: usize,
    jlpt_content: &JlptContent,
    rng: &mut R,
    now: DateTime<Utc>,
) {
    // Compute `allowed` BEFORE distribute so the per-type slot allocator
    // knows the actual quota (otherwise it would build an unbounded list
    // and we'd slice the tail off, defeating the proportional split).
    let available = MAX_LESSON_SIZE.saturating_sub(selected_cards.len() + favorite_ids.len());
    let daily_remaining = daily_new_remaining.saturating_sub(
        selected_cards
            .iter()
            .filter(|(_, c)| c.memory().is_new())
            .count(),
    );
    let allowed = daily_remaining.min(available);

    if allowed == 0 {
        return;
    }

    let new_core_cards: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            is_core_candidate(id, card, favorite_ids, now) && card.memory().is_new()
        })
        .copied()
        .collect();

    if new_core_cards.is_empty() {
        return;
    }

    let distributed = distribute_new_cards(new_core_cards, jlpt_content, allowed, rng);
    selected_cards.extend(distributed);
}

pub(super) fn fill_core_due_known<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    selected_cards: &mut Vec<(&'a Ulid, &'a StudyCard)>,
    favorite_ids: &HashSet<Ulid>,
    now: DateTime<Utc>,
) {
    let current_count = selected_cards.len() + favorite_ids.len();
    let remaining = MAX_LESSON_SIZE.saturating_sub(current_count);
    if remaining == 0 {
        return;
    }

    let due_known: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            is_core_candidate(id, card, favorite_ids, now)
                && card.memory().is_due()
                && (card.memory().is_in_progress() || card.memory().is_known_card())
        })
        .take(remaining)
        .copied()
        .collect();
    selected_cards.extend(due_known);
}

pub(super) fn collect_padding<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    all_selected_ids: &HashSet<Ulid>,
    now: DateTime<Utc>,
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
                && !card.memory().has_active_ghost(now)
                && card.memory().is_high_difficulty()
        })
        .copied()
        .collect();
    candidates.sort_by_key(|(_, card)| card.memory().next_review_date());
    candidates.into_iter().take(needed).collect()
}

pub(super) fn build_selected_ids(
    selected_cards: &[(&Ulid, &StudyCard)],
    favorite_cards: &[(&Ulid, &StudyCard)],
) -> HashSet<Ulid> {
    let selected_ids: HashSet<_> = selected_cards.iter().map(|(id, _)| **id).collect();
    let favorite_ids: HashSet<_> = favorite_cards.iter().map(|(id, _)| **id).collect();
    selected_ids.union(&favorite_ids).copied().collect()
}

pub(super) fn build_core_lesson_cards(
    favorite_cards: &[(&Ulid, &StudyCard)],
    selected_cards: &[(&Ulid, &StudyCard)],
    padding_cards: &[(&Ulid, &StudyCard)],
    knowledge_set: &KnowledgeSet,
    native_language: NativeLanguage,
) -> Vec<(Ulid, LessonCard)> {
    let padding_ids: HashSet<_> = padding_cards.iter().map(|(id, _)| **id).collect();
    let mut generator = LessonViewGenerator::new(knowledge_set, native_language);

    let favorite_lessons: Vec<_> = favorite_cards
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            let is_short_term = padding_ids.contains(card_id);
            (**card_id, LessonCard::new(**card_id, view, is_short_term))
        })
        .collect();

    let selected_lessons: Vec<_> = selected_cards
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            let is_short_term = padding_ids.contains(card_id);
            (**card_id, LessonCard::new(**card_id, view, is_short_term))
        })
        .collect();

    let padding_lessons: Vec<_> = padding_cards
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            (**card_id, LessonCard::new(**card_id, view, true))
        })
        .collect();

    let mut result = favorite_lessons;
    result.extend(selected_lessons);
    result.extend(padding_lessons);
    result
}
