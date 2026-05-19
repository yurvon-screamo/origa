# ADR-003: Рекалибровка порогов категоризации карточек FSRS

## Status

Accepted

## Date

2026-05-19

## Context

Приложение Origa categorizes study cards into three statuses based on FSRS metrics:

- **Difficult** (сложные) — cards that are hard to remember
- **In Progress** (в процессе) — cards being actively learned  
- **Learned** (изучено) — cards that are well-memorized

The original thresholds caused a severe distribution skew: ~100 cards stuck in "Difficult", only ~4 in "In Progress". The root cause was `HIGH_DIFFICULTY_THRESHOLD = 5.0` which captured cards with a normal "Good" rating (FSRS initial difficulty D₀(Good) ≈ 5.31).

### FSRS Difficulty parameter reference

- Range: [1.0, 10.0]
- Initial difficulty by first rating: Again ≈ 7.21, Hard ≈ 6.51, Good ≈ 5.31, Easy ≈ 3.28
- "Good" does not change difficulty; "Easy" reduces by ~1.07; "Again" increases by ~2.13
- Mean reversion (w₇ = 0.0234) is extremely slow — takes dozens of reviews to shift difficulty
- Difficulty affects stability growth via multiplier (11 - D)

### Original thresholds (problematic)

- `HIGH_DIFFICULTY_THRESHOLD: f64 = 5.0`
- `MAX_DAYS_INTERVAL_THRESHOLD: i64 = 10` (used `latest_interval`)
- `KNOWN_CARD_STABILITY_THRESHOLD: f64 = 10.0`

## Decision

### New thresholds

- `HIGH_DIFFICULTY_THRESHOLD: f64 = 7.0` — only captures Again-rated and chronically difficult cards
- `HIGH_DIFFICULTY_STABILITY_CAP: f64 = 7.0` — if stability > 7 days, card has progressed despite difficulty
- `KNOWN_CARD_STABILITY_THRESHOLD: f64 = 21.0` — card repeating less than once per 3 weeks is truly learned

### Rewrite `is_high_difficulty()`

Replaced `latest_interval` (unstable artifact of last review) with `stability` (invariant FSRS memory metric). New logic: D ≥ 7.0 AND S < 7.0.

### Removed

- `MAX_DAYS_INTERVAL_THRESHOLD` — replaced by `HIGH_DIFFICULTY_STABILITY_CAP`
- `latest_interval()` method — dead code after the change

## Alternatives Considered

### Use retrievability as primary metric

Retrievability R(t, S) = (1 + 19/81 * t/S)^(-0.5) is the probability of recall. Using R as a categorization metric is theoretically more correct, but adds runtime computation (requires elapsed_days calculation) and is less intuitive for threshold tuning.

### Use FSRS State enum (New/Learning/Review/Relearning)

FSRS provides card states, but they don't capture "difficulty" semantics. A card in "Review" state can still be very difficult. Our categorization adds value on top of FSRS states.

### Keep difficulty threshold at 5.0 but remove interval check

Would still capture most cards (D₀(Good)=5.31), only marginally better than original.

## Consequences

- Expected distribution shift: ~15-20% Difficult, ~50-60% In Progress, ~20-30% Learned (vs. ~96%/4%/0%)
- Hard rating no longer makes a card "Difficult" — only multiple Again/Hard ratings do. This is semantically correct: Hard ≠ problematic card.
- Cards with stability 10-21 migrate from "Learned" to "In Progress" — more accurate representation
- Statistics dashboard will show a one-time jump when daily history recalculates
- `is_known_card()` affects JLPT progress counting, phrase seeding, and content scoring — these will use stricter criteria, which is correct
