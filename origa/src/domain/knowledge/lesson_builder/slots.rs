use super::*;

/// Приоритет карточек без определённого JLPT уровня — ниже всех известных уровней (N1=1)
pub(super) const UNKNOWN_JLPT_PRIORITY: u8 = 0;

/// Веса типов карточек для interleaving и состава руки знакомства:
/// Vocab:Kanji:Grammar:Counter ≈ 80:10:10:10 (issue #415 — Counter
/// добавлен четвёртым типом; доля слов при полном пуле ~72.7%).
/// При добавлении нового варианта в CardType — обновить эту константу.
pub(super) const CARD_TYPE_WEIGHTS: [(CardType, usize); 4] = [
    (CardType::Vocabulary, 8),
    (CardType::Kanji, 1),
    (CardType::Grammar, 1),
    (CardType::Counter, 1),
];

pub(super) fn resolve_jlpt_level(card: &Card, jlpt_content: &JlptContent) -> Option<JapaneseLevel> {
    jlpt_content.find_level(&card.content_key(), CardType::from(card))
}

/// Единая политика сортировочного приоритета новых карт: номер JLPT-уровня
/// (N5=5 .. N1=1); карты без известного уровня — в самом конце пула.
/// Используется и билдером урока, и выбором руки знакомства (CN3).
pub(crate) fn jlpt_sort_key(card: &Card, jlpt_content: &JlptContent) -> u8 {
    resolve_jlpt_level(card, jlpt_content)
        .map(|level| level.as_number())
        .unwrap_or(UNKNOWN_JLPT_PRIORITY)
}

/// Distributes `allowed` slots across card types proportionally to
/// `CARD_TYPE_WEIGHTS`, treating the weights as **percentages** (not as a
/// round-robin pattern). Uses the largest-remainder method with a
/// minor-priority rule for leftover slots:
///
/// 1. `raw_t = allowed * w_t / sum_w` (where `sum_w` is **renormalized** over
///    types actually present in `available_by_type`).
/// 2. `floor_t = min(floor(raw_t), available_t)`.
/// 3. `leftover = allowed - sum(floor_t)`.
/// 4. Phase 1 (minor types, non-Vocabulary): candidates with `floor_t <
///    available_t` AND `floor_t < ceil(raw_t)`; sorted by `fract(raw_t)` desc
///    with a random tie-break.
/// 5. Phase 2 (fallback): Vocabulary first, then any remaining type, until
///    leftover is exhausted or availability runs out.
///
/// This fixes the systematic grammar starvation at `daily_load ≤ 9` where the
/// old round-robin pattern (`8V + 1K + 0G`) gave grammar 0 slots every day.
pub(super) fn compute_type_slots<R: rand::Rng>(
    allowed: usize,
    available_by_type: &HashMap<CardType, usize>,
    rng: &mut R,
) -> HashMap<CardType, usize> {
    if allowed == 0 {
        return HashMap::new();
    }

    // Active types: CARD_TYPE_WEIGHTS entries with available > 0.
    // If a type is missing from the pool, sum_w is renormalized without it.
    let active: Vec<(CardType, usize)> = CARD_TYPE_WEIGHTS
        .iter()
        .copied()
        .filter(|(t, _)| available_by_type.get(t).copied().unwrap_or(0) > 0)
        .collect();
    if active.is_empty() {
        return HashMap::new();
    }

    let sum_w: usize = active.iter().map(|(_, w)| w).sum();

    // raw and floor (with availability cap).
    let mut slots: HashMap<CardType, usize> = HashMap::new();
    let mut raw_map: HashMap<CardType, f64> = HashMap::new();
    let mut sum_floor = 0usize;
    for (t, w) in &active {
        let raw = (allowed as f64) * (*w as f64) / (sum_w as f64);
        raw_map.insert(*t, raw);
        let avail = available_by_type.get(t).copied().unwrap_or(0);
        let floor = (raw.floor() as usize).min(avail);
        slots.insert(*t, floor);
        sum_floor += floor;
    }

    let mut leftover = allowed.saturating_sub(sum_floor);
    if leftover == 0 {
        debug_assert!(slots.values().sum::<usize>() <= allowed);
        return slots;
    }

    // Phase 1: minor (non-Vocabulary) types first.
    // Candidate must still have headroom: floor < available AND floor < ceil(raw).
    let mut minor_candidates: Vec<CardType> = active
        .iter()
        .filter(|(t, _)| *t != CardType::Vocabulary)
        .map(|(t, _)| *t)
        .filter(|t| {
            let cur = slots.get(t).copied().unwrap_or(0);
            let avail = available_by_type.get(t).copied().unwrap_or(0);
            let raw = raw_map.get(t).copied().unwrap_or(0.0);
            cur < avail && cur < raw.ceil() as usize
        })
        .collect();

    // Shuffle then stable-sort by remainder desc: ties keep the random order
    // from the shuffle, giving the random tie-break required by the contract.
    minor_candidates.shuffle(rng);
    minor_candidates.sort_by(|a, b| {
        let rem_a = raw_map.get(a).copied().unwrap_or(0.0).fract();
        let rem_b = raw_map.get(b).copied().unwrap_or(0.0).fract();
        rem_b
            .partial_cmp(&rem_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for t in minor_candidates.iter() {
        if leftover == 0 {
            break;
        }
        let cur = slots.get(t).copied().unwrap_or(0);
        let avail = available_by_type.get(t).copied().unwrap_or(0);
        if cur < avail {
            slots.insert(*t, cur + 1);
            leftover -= 1;
        }
    }

    // Phase 2: Vocabulary first, then any remaining type with headroom.
    if leftover > 0 {
        let phase2_order: Vec<CardType> = {
            let mut v_first: Vec<CardType> = active
                .iter()
                .filter(|(t, _)| *t == CardType::Vocabulary)
                .map(|(t, _)| *t)
                .collect();
            let mut rest: Vec<CardType> = active
                .iter()
                .filter(|(t, _)| *t != CardType::Vocabulary)
                .map(|(t, _)| *t)
                .collect();
            v_first.append(&mut rest);
            v_first
        };
        for t in phase2_order {
            if leftover == 0 {
                break;
            }
            let avail = available_by_type.get(&t).copied().unwrap_or(0);
            while leftover > 0 {
                let cur = slots.get(&t).copied().unwrap_or(0);
                if cur >= avail {
                    break;
                }
                slots.insert(t, cur + 1);
                leftover -= 1;
            }
        }
    }

    debug_assert!(slots.values().sum::<usize>() <= allowed);
    slots
}

/// New cards grouped first by JLPT priority (N5 highest) then by CardType.
/// Used by [`distribute_new_cards`] to walk groups in priority order while
/// keeping per-type queues for the proportional slot allocator.
pub(super) type GroupedNewCards<'a> =
    BTreeMap<Reverse<u8>, HashMap<CardType, VecDeque<(&'a Ulid, &'a StudyCard)>>>;

/// Picks the new cards for the lesson respecting both the JLPT-level priority
/// (N5 first, then N4, …) and the per-type proportional quota derived from
/// `CARD_TYPE_WEIGHTS`. `allowed` bounds the total returned; each JLPT group
/// consumes up to `min(remaining, group_size)` slots, allocated across types
/// via `compute_type_slots`. Output order: Vocabulary as spine, then Kanji,
/// then Grammar (matches the historical lesson layout — `interleave_core_by_type`
/// reshuffles it later anyway).
pub(crate) fn distribute_new_cards<'a, R: rand::Rng>(
    new_cards: Vec<(&'a Ulid, &'a StudyCard)>,
    jlpt_content: &JlptContent,
    allowed: usize,
    rng: &mut R,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    debug_assert!(
        new_cards
            .iter()
            .all(|(_, sc)| !matches!(sc.card(), Card::Phrase(_))),
        "phrases must be filtered out before distribute_new_cards (is_core_candidate)"
    );

    if allowed == 0 || new_cards.is_empty() {
        return Vec::new();
    }

    // Reverse: N5(5) → highest priority → first key in BTreeMap.
    let mut groups: GroupedNewCards = BTreeMap::new();
    for card in new_cards {
        let priority = jlpt_sort_key(card.1.card(), jlpt_content);
        groups
            .entry(Reverse(priority))
            .or_default()
            .entry(CardType::from(card.1.card()))
            .or_default()
            .push_back(card);
    }

    let mut result: Vec<(&Ulid, &StudyCard)> = Vec::with_capacity(allowed);
    let mut remaining = allowed;
    for (_, by_type) in groups {
        if remaining == 0 {
            break;
        }
        let group_total: usize = by_type.values().map(|q| q.len()).sum();
        let take = remaining.min(group_total);
        if take == 0 {
            continue;
        }

        let available: HashMap<CardType, usize> =
            by_type.iter().map(|(t, q)| (*t, q.len())).collect();
        let slots = compute_type_slots(take, &available, rng);
        // `take` is clamped to `group_total` above, so `compute_type_slots`
        // must allocate exactly `take` slots. Guards against a future
        // allocator change that would silently shorten `result`.
        debug_assert_eq!(
            slots.values().sum::<usize>(),
            take,
            "compute_type_slots must allocate exactly `take` slots"
        );

        for card_type in [
            CardType::Vocabulary,
            CardType::Kanji,
            CardType::Grammar,
            CardType::Counter,
        ] {
            if let Some(queue) = by_type.get(&card_type) {
                let n = slots.get(&card_type).copied().unwrap_or(0);
                for card in queue.iter().take(n) {
                    result.push(*card);
                }
            }
        }

        remaining -= take;
    }

    result
}
