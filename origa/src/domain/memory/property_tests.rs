//! Property-based tests for the FSRS wrapper (issue #802).
//!
//! The examples in `mod.rs` and `ghost/tests.rs` pin single cases; these
//! properties pin the INVARIANTS of the wrapper over random rating chains:
//! counters, merge semantics, the ghost state machine, and the scheduling
//! boundary (the wrapper must never hand a corrupt state back to the app).
//!
//! Scope: the wrapper only. `rs-fsrs` tests its own algorithm — the
//! properties here describe what OUR code guarantees around it.

use chrono::{DateTime, Duration, Utc};
use proptest::prelude::*;
use ulid::Ulid;

use super::{CardState, Difficulty, MemoryHistory, MemoryState, Rating, Stability};
use crate::domain::RateMode;
use crate::domain::srs::rate_memory;

use super::GhostRung;
use super::GhostState;

/// Every scheduling mode of the app: the wrapper configures each one
/// differently (retention, maximum interval, fuzz), so the scheduling
/// properties run over all of them.
const ALL_RATE_MODES: [RateMode; 7] = [
    RateMode::ShortTerm,
    RateMode::StandardLesson,
    RateMode::PhraseReview,
    RateMode::OnboardingScoring,
    RateMode::GrammarReview,
    RateMode::KanjiReview,
    RateMode::CounterReview,
];

/// Uniform strategy over the four ratings.
fn rating_strategy() -> impl Strategy<Value = Rating> {
    (0u8..4).prop_map(|n| match n {
        0 => Rating::Again,
        1 => Rating::Hard,
        2 => Rating::Good,
        _ => Rating::Easy,
    })
}

/// Valid stability: FSRS divides by it, so it must be strictly positive.
/// Zero is a separate, deliberately degenerate case (see the finiteness
/// property).
fn valid_stability_strategy() -> impl Strategy<Value = f64> {
    0.1f64..50.0
}

/// Valid difficulty: the FSRS range is [1, 10].
fn valid_difficulty_strategy() -> impl Strategy<Value = f64> {
    1.0f64..10.0
}

/// A review state the scheduler can work with: positive stability,
/// difficulty inside the FSRS range, due now.
fn valid_review_state(stability: f64, difficulty: f64, due: DateTime<Utc>) -> MemoryState {
    MemoryState::with_card_state(
        Stability::new(stability).expect("stability is finite and positive"),
        Difficulty::new(difficulty).expect("difficulty is inside the FSRS range"),
        due,
        CardState::Review,
    )
}

/// History that already carries the given reviews (counters included).
fn history_after(ratings: &[Rating]) -> MemoryHistory {
    let mut history = MemoryHistory::new();
    let state = MemoryState::new(
        Stability::new(5.0).expect("5.0 is a valid stability"),
        Difficulty::new(3.0).expect("3.0 is a valid difficulty"),
        Utc::now(),
    );
    for rating in ratings {
        history.apply_review(state.clone(), *rating);
    }
    history
}

/// Model of the ghost state machine (`domain/memory/ghost.rs`): the ghost
/// state and the failure counter after one rating.
///
/// A success resets the failure counter and advances the ladder when the
/// window is open (the third rung resolves the ghost). A failure restarts
/// the ladder at the first rung when the window is open; without an active
/// ghost the failures accumulate and the second one spawns a new cycle —
/// a terminal ghost is replaced by that spawn.
fn model_ghost_step(
    current: Option<&GhostState>,
    consecutive: u8,
    rating: Rating,
    now: DateTime<Utc>,
) -> (Option<GhostState>, u8) {
    let restarted = || {
        Some(GhostState::Active {
            rung: GhostRung::First,
            due_at: now + GhostRung::First.interval(),
            last_transition_at: now,
        })
    };

    match rating {
        Rating::Good | Rating::Easy => {
            let mut next_state = current.cloned();
            if let Some(GhostState::Active { rung, due_at, .. }) = &next_state {
                if *due_at <= now {
                    next_state = Some(match rung.next() {
                        Some(next) => GhostState::Active {
                            rung: next,
                            due_at: now + next.interval(),
                            last_transition_at: now,
                        },
                        None => GhostState::Resolved { at: now },
                    });
                }
            }
            (next_state, 0)
        },
        Rating::Again | Rating::Hard => {
            if let Some(GhostState::Active { due_at, .. }) = current {
                return if *due_at <= now {
                    (restarted(), consecutive)
                } else {
                    (current.cloned(), consecutive)
                };
            }
            let counter = consecutive.saturating_add(1);
            if counter >= 2 {
                (restarted(), 0)
            } else {
                (current.cloned(), counter)
            }
        },
    }
}

proptest! {
    /// The counters are a projection of the rating chain: nothing else may
    /// touch them.
    #[test]
    fn counters_track_the_applied_rating_chain(
        ratings in prop::collection::vec(rating_strategy(), 0..40),
    ) {
        let mut history = MemoryHistory::new();
        let state = MemoryState::new(
            Stability::new(5.0).expect("5.0 is a valid stability"),
            Difficulty::new(3.0).expect("3.0 is a valid difficulty"),
            Utc::now(),
        );

        let mut reps = 0u32;
        let mut lapses = 0u32;
        let mut easy = 0usize;
        let mut good = 0usize;
        for rating in &ratings {
            history.apply_review(state.clone(), *rating);
            reps += 1;
            match rating {
                Rating::Again => lapses += 1,
                Rating::Easy => easy += 1,
                Rating::Good => good += 1,
                Rating::Hard => {},
            }
        }

        prop_assert_eq!(history.reps(), reps);
        prop_assert_eq!(history.lapses(), lapses);
        prop_assert_eq!(history.easy_review_count(), easy);
        prop_assert_eq!(history.good_review_count(), good);
        prop_assert_eq!(history.last_rating(), ratings.last().copied());
    }

    /// Cross-device merge keeps the higher counter of each side (the
    /// documented max() semantics of the merge).
    #[test]
    fn merge_never_lowers_the_counters(
        left_ratings in prop::collection::vec(rating_strategy(), 0..20),
        right_ratings in prop::collection::vec(rating_strategy(), 0..20),
    ) {
        let left = history_after(&left_ratings);
        let right = history_after(&right_ratings);

        let mut merged = left.clone();
        merged.merge(&right);

        prop_assert!(merged.reps() >= left.reps());
        prop_assert!(merged.reps() >= right.reps());
        prop_assert!(merged.lapses() >= left.lapses());
        prop_assert!(merged.lapses() >= right.lapses());
        prop_assert!(merged.easy_review_count() >= left.easy_review_count());
        prop_assert!(merged.easy_review_count() >= right.easy_review_count());
        prop_assert!(merged.good_review_count() >= left.good_review_count());
        prop_assert!(merged.good_review_count() >= right.good_review_count());
    }

    /// Merging the same side twice changes nothing (the merge is idempotent).
    #[test]
    fn merge_applied_twice_keeps_the_same_counters(
        left_ratings in prop::collection::vec(rating_strategy(), 0..20),
        right_ratings in prop::collection::vec(rating_strategy(), 0..20),
    ) {
        let left = history_after(&left_ratings);
        let right = history_after(&right_ratings);

        let mut merged = left.clone();
        merged.merge(&right);
        let once = merged.clone();
        merged.merge(&right);

        prop_assert_eq!(merged.reps(), once.reps());
        prop_assert_eq!(merged.lapses(), once.lapses());
        prop_assert_eq!(merged.easy_review_count(), once.easy_review_count());
        prop_assert_eq!(merged.good_review_count(), once.good_review_count());
    }

    /// The ghost spawns only after two consecutive failures (Again or
    /// Hard). Precondition: no active ghost before the chain (an active
    /// ghost short-circuits the counter) and a card that was not new
    /// before the rating.
    #[test]
    fn ghost_spawns_only_after_two_consecutive_failures(
        failures in prop::collection::vec(any::<bool>(), 0..20),
    ) {
        let now = Utc::now();
        let card_id = Ulid::new();
        let mut history = MemoryHistory::new();
        history.seed(valid_review_state(5.0, 5.0, now));

        for (index, is_hard) in failures.iter().enumerate() {
            let rating = if *is_hard { Rating::Hard } else { Rating::Again };
            history.apply_ghost_transition(rating, now, false, &card_id);

            if index < 1 {
                prop_assert!(
                    history.ghost().is_none(),
                    "no ghost may exist before the second consecutive failure"
                );
                prop_assert_eq!(history.consecutive_again(), (index + 1) as u8);
            } else {
                prop_assert!(
                    history.ghost().is_some(),
                    "the ghost spawns on the second consecutive failure"
                );
                prop_assert_eq!(history.consecutive_again(), 0);
            }
        }

        // The new-card gate: a card that was new before the rating never
        // spawns a ghost, no matter how many failures it collects.
        let mut new_card = MemoryHistory::new();
        new_card.apply_ghost_transition(Rating::Again, now, true, &card_id);
        new_card.apply_ghost_transition(Rating::Again, now, true, &card_id);
        prop_assert!(
            new_card.ghost().is_none(),
            "a card that was new before the rating must not spawn a ghost"
        );
    }

    /// The ghost state machine: a success in an open window advances the
    /// rung (the last rung resolves the ghost), a failure in an open window
    /// restarts the ladder at the first rung, and a closed window freezes
    /// the ghost.
    #[test]
    fn ghost_advances_the_rung_only_on_success_in_an_open_window(
        rung in prop::sample::select(vec![
            GhostRung::First,
            GhostRung::Second,
            GhostRung::Third,
        ]),
        window_open in any::<bool>(),
        ratings in prop::collection::vec(rating_strategy(), 0..20),
    ) {
        let now = Utc::now();
        let card_id = Ulid::new();
        let due_at = if window_open {
            now - Duration::hours(1)
        } else {
            now + Duration::hours(1)
        };

        let mut history = MemoryHistory::new();
        history.seed(valid_review_state(5.0, 5.0, now));
        history.set_ghost_for_test(Some(GhostState::active(rung, due_at, now)));

        let mut expected = Some(GhostState::Active {
            rung,
            due_at,
            last_transition_at: now,
        });
        let mut expected_consecutive = 0u8;
        for rating in &ratings {
            let (next_state, next_consecutive) =
                model_ghost_step(expected.as_ref(), expected_consecutive, *rating, now);

            history.apply_ghost_transition(*rating, now, false, &card_id);
            prop_assert_eq!(history.ghost(), next_state.as_ref());
            prop_assert_eq!(history.consecutive_again(), next_consecutive);

            expected = next_state;
            expected_consecutive = next_consecutive;
        }
    }

    /// The TTL is lazy: a ghost that has been silent for longer than the
    /// TTL is written off as expired on the next rating.
    #[test]
    fn ghost_becomes_expired_after_the_ttl_on_the_next_rating(
        rating in rating_strategy(),
        age_days in 31i64..400,
    ) {
        let now = Utc::now();
        let card_id = Ulid::new();
        let mut history = MemoryHistory::new();
        history.seed(valid_review_state(5.0, 5.0, now));
        history.set_ghost_for_test(Some(GhostState::active(
            GhostRung::First,
            now - Duration::days(1),
            now - Duration::days(age_days),
        )));

        history.apply_ghost_transition(rating, now, false, &card_id);

        prop_assert!(
            matches!(history.ghost(), Some(GhostState::Expired { at }) if *at == now),
            "a ghost older than the TTL must expire on the next rating"
        );
    }

    /// The merge keeps the ghost with the later transition; a side without a
    /// ghost never erases the other side.
    #[test]
    fn merge_keeps_the_ghost_with_the_later_transition(
        left_age_days in prop::option::of(0i64..100),
        right_age_days in prop::option::of(0i64..100),
    ) {
        let now = Utc::now();
        let left = history_after(&[]);
        let right = history_after(&[]);

        let left_ghost = left_age_days.map(|age| {
            GhostState::active(
                GhostRung::First,
                now - Duration::days(1),
                now - Duration::days(age),
            )
        });
        let right_ghost = right_age_days.map(|age| {
            GhostState::active(
                GhostRung::First,
                now - Duration::days(1),
                now - Duration::days(age),
            )
        });

        let mut left = left;
        left.set_ghost_for_test(left_ghost.clone());
        let mut right = right;
        right.set_ghost_for_test(right_ghost.clone());

        let expected = match (&left_ghost, &right_ghost) {
            (None, None) => None,
            (Some(_), None) => left_ghost.clone(),
            (None, Some(_)) => right_ghost.clone(),
            (Some(left), Some(right)) => {
                if right.last_transition_at() >= left.last_transition_at() {
                    right_ghost.clone()
                } else {
                    left_ghost.clone()
                }
            },
        };

        let mut merged = left.clone();
        merged.merge(&right);
        prop_assert_eq!(merged.ghost(), expected.as_ref());
    }

    /// The scheduler never fails on a state it is able to work with: every
    /// mode turns a valid history into the next state.
    #[test]
    fn rate_memory_schedules_any_rating_chain_without_error(
        stability in valid_stability_strategy(),
        difficulty in valid_difficulty_strategy(),
        ratings in prop::collection::vec(rating_strategy(), 0..12),
    ) {
        let mut history = MemoryHistory::new();
        // The chain starts from a state the scheduler can work with.
        history.seed(valid_review_state(stability, difficulty, Utc::now()));
        for rating in &ratings {
            for mode in ALL_RATE_MODES {
                let next = rate_memory(mode, *rating, &history);
                prop_assert!(
                    next.is_ok(),
                    "scheduling failed in {mode:?} for rating {rating:?}: {:?}",
                    next.err()
                );
            }
            let next = rate_memory(RateMode::StandardLesson, *rating, &history)
                .expect("the standard lesson mode schedules every valid state");
            history.apply_review(next, *rating);
        }
    }

    /// The scheduling boundary: the wrapper never hands a corrupt state
    /// back. A degenerate input (zero stability in a review state) makes
    /// rs-fsrs compute `elapsed / stability` — the result must be an error,
    /// never a NaN stability.
    #[test]
    fn scheduled_memory_values_stay_finite_and_non_negative(
        stability in prop_oneof![2 => Just(0.0f64), 1 => valid_stability_strategy()],
        difficulty in valid_difficulty_strategy(),
        rating in rating_strategy(),
    ) {
        let now = Utc::now();
        let mut history = MemoryHistory::new();
        history.seed(MemoryState::with_card_state(
            Stability::new(stability).expect("stability is finite"),
            Difficulty::new(difficulty).expect("difficulty is inside the FSRS range"),
            now,
            CardState::Review,
        ));
        // The last review happened today: FSRS then divides the elapsed
        // time (zero days) by the stability — the degenerate case.
        history.last_review_date = Some(now);

        for mode in ALL_RATE_MODES {
            if let Ok(next) = rate_memory(mode, rating, &history) {
                prop_assert!(
                    next.stability().value().is_finite(),
                    "stability must be finite in {mode:?}, got {}",
                    next.stability().value()
                );
                prop_assert!(
                    next.difficulty().value().is_finite(),
                    "difficulty must be finite in {mode:?}, got {}",
                    next.difficulty().value()
                );
                prop_assert!(next.stability().value() >= 0.0);
                prop_assert!(next.difficulty().value() >= 0.0);
            }
        }
    }

    /// A Good series never lowers the stability. The reviews are spread over
    /// time (the last review is moved into the past): with every review on
    /// the same instant the retrievability is 1 and the stability freezes
    /// after the first step, so the property would check nothing.
    #[test]
    fn good_series_never_lowers_the_stability(
        stability in valid_stability_strategy(),
        difficulty in valid_difficulty_strategy(),
        rounds in 1u32..8,
        elapsed_days in 1i64..30,
    ) {
        let mut history = MemoryHistory::new();
        let mut previous: Option<f64> = None;

        for _ in 0..rounds {
            let now = Utc::now();
            history.seed(valid_review_state(stability, difficulty, now));
            history.last_review_date = Some(now - Duration::days(elapsed_days));

            for mode in ALL_RATE_MODES {
                let next = rate_memory(mode, Rating::Good, &history)
                    .expect("a valid state schedules in every mode");
                let current = next.stability().value();
                prop_assert!(
                    current.is_finite(),
                    "stability must be finite in {mode:?}"
                );
                if let Some(previous) = previous {
                    prop_assert!(
                        current >= previous,
                        "a Good series must not lower the stability in {mode:?}: {previous} -> {current}"
                    );
                }
                previous = Some(current);
            }
        }
    }
}

/// Regression guard for the degenerate input the finiteness property
/// explores: a review card with zero stability must be rejected by the
/// scheduler instead of producing a NaN stability.
#[test]
fn rate_memory_rejects_a_zero_stability_review_state() {
    let now = Utc::now();
    let mut history = MemoryHistory::new();
    history.seed(MemoryState::with_card_state(
        Stability::new(0.0).expect("zero is a finite stability"),
        Difficulty::new(5.0).expect("5.0 is a valid difficulty"),
        now,
        CardState::Review,
    ));
    history.last_review_date = Some(now);

    for mode in ALL_RATE_MODES {
        for rating in [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy] {
            assert!(
                rate_memory(mode, rating, &history).is_err(),
                "a zero-stability review card must not schedule in {mode:?} for {rating:?}"
            );
        }
    }
}
