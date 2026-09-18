use chrono::{Duration, TimeDelta, Utc};
use rstest::rstest;
use ulid::Ulid;

use super::super::{GhostRung, GhostState, MemoryHistory, Rating};

fn t0() -> chrono::DateTime<Utc> {
    Utc::now()
}

/// Не-новая карта: текущее состояние есть, добивания нет.
fn seasoned_history() -> MemoryHistory {
    let mut history = MemoryHistory::new();
    let past = t0() - Duration::days(10);
    history.seed(super::super::MemoryState::new(
        super::super::Stability::new(5.0).unwrap(),
        super::super::Difficulty::new(5.0).unwrap(),
        past,
    ));
    history
}

fn active(rung: GhostRung, due_at: chrono::DateTime<Utc>) -> MemoryHistory {
    let mut history = seasoned_history();
    history.ghost = Some(GhostState::active(rung, due_at, t0() - Duration::hours(1)));
    history
}

fn ghost_of(history: &MemoryHistory) -> &GhostState {
    history.ghost.as_ref().expect("ghost must be present")
}

fn ghost_due_at(history: &MemoryHistory) -> chrono::DateTime<Utc> {
    *ghost_of(history)
        .due_at()
        .expect("active ghost must have a due_at")
}

fn rate(history: &mut MemoryHistory, rating: Rating, now: chrono::DateTime<Utc>) {
    history.apply_ghost_transition(rating, now, false, &Ulid::new());
}

// --- спавн ---

#[test]
fn second_consecutive_again_spawns_ghost_at_first_rung() {
    // Arrange
    let mut history = seasoned_history();
    let now = t0();

    // Act
    rate(&mut history, Rating::Again, now);
    rate(&mut history, Rating::Again, now);

    // Assert
    let ghost = ghost_of(&history);
    let GhostState::Active { rung, due_at, .. } = ghost else {
        panic!("ghost must be active, got {ghost:?}");
    };
    assert_eq!(*rung, GhostRung::First);
    assert_eq!(*due_at, now + Duration::hours(12));
}

#[test]
fn first_again_on_new_card_spawns_nothing() {
    // Arrange
    let mut history = MemoryHistory::new();
    let now = t0();

    // Act
    history.apply_ghost_transition(Rating::Again, now, true, &Ulid::new());

    // Assert
    assert_eq!(history.ghost(), None);
    assert_eq!(history.consecutive_again(), 1);
}

#[test]
fn good_between_two_agains_resets_consecutive_counter() {
    // Arrange
    let mut history = seasoned_history();
    let now = t0();

    // Act
    rate(&mut history, Rating::Again, now);
    rate(&mut history, Rating::Good, now);
    rate(&mut history, Rating::Again, now);

    // Assert
    assert_eq!(
        history.ghost(),
        None,
        "one Again after a Good must not spawn"
    );
    assert_eq!(history.consecutive_again(), 1);
}

// --- лестница и закрытие ---

#[rstest]
#[case::first_to_second(GhostRung::First, GhostRung::Second)]
#[case::second_to_third(GhostRung::Second, GhostRung::Third)]
fn good_on_open_window_advances_to_next_rung(#[case] from: GhostRung, #[case] expected: GhostRung) {
    // Arrange: окно открыто (due в прошлом)
    let mut history = active(from, t0() - Duration::hours(1));
    let now = t0();

    // Act
    rate(&mut history, Rating::Good, now);

    // Assert
    let GhostState::Active { rung, due_at, .. } = ghost_of(&history) else {
        panic!("ghost must stay active");
    };
    assert_eq!(*rung, expected);
    let expected_due = now + expected.interval();
    assert_eq!(*due_at, expected_due);
}

#[test]
fn third_consecutive_good_resolves_ghost() {
    // Arrange: два успеха уже накоплены (ступень Third), окно открыто
    let mut history = active(GhostRung::Third, t0() - Duration::hours(1));

    // Act
    rate(&mut history, Rating::Good, t0());

    // Assert
    assert!(
        matches!(ghost_of(&history), GhostState::Resolved { .. }),
        "third consecutive success must resolve"
    );
}

#[test]
fn success_when_window_closed_is_ignored() {
    // Arrange: окно откроется только завтра
    let mut history = active(GhostRung::Second, t0() + Duration::days(1));
    let ghost_before = history.ghost().cloned();

    // Act
    rate(&mut history, Rating::Good, t0());

    // Assert
    assert_eq!(history.ghost(), ghost_before.as_ref());
}

#[rstest]
#[case::easy_counts_as_success(Rating::Easy, true)]
#[case::good_counts_as_success(Rating::Good, true)]
#[case::hard_counts_as_failure(Rating::Hard, false)]
#[case::again_counts_as_failure(Rating::Again, false)]
fn easy_counts_as_success_and_hard_counts_as_failure(
    #[case] rating: Rating,
    #[case] is_success: bool,
) {
    // Arrange: ступень Second, окно открыто
    let mut history = active(GhostRung::Second, t0() - Duration::hours(1));

    // Act
    rate(&mut history, rating, t0());

    // Assert: успех продвигает на Third, провал рестартует на First
    let GhostState::Active { rung, .. } = ghost_of(&history) else {
        panic!("ghost must stay active");
    };
    let expected = if is_success {
        GhostRung::Third
    } else {
        GhostRung::First
    };
    assert_eq!(*rung, expected);
}

#[test]
fn again_at_second_rung_resets_streak_and_restarts_from_first_rung() {
    // Arrange: два успеха накоплены (Third), окно открыто
    let mut history = active(GhostRung::Third, t0() - Duration::hours(1));
    let now = t0();

    // Act
    rate(&mut history, Rating::Again, now);

    // Assert
    let GhostState::Active { rung, due_at, .. } = ghost_of(&history) else {
        panic!("ghost must stay active after a restart");
    };
    assert_eq!(*rung, GhostRung::First);
    assert_eq!(*due_at, now + Duration::hours(12));
}

#[test]
fn ladder_never_skips_or_exceeds_third_rung() {
    // Arrange: три успеха подряд с закрытием окон между шагами
    let mut history = active(GhostRung::First, t0() - Duration::hours(1));

    // Act
    rate(&mut history, Rating::Good, t0());
    let due = ghost_due_at(&history);
    rate(&mut history, Rating::Good, due);
    let due = ghost_due_at(&history);
    rate(&mut history, Rating::Good, due);

    // Assert: First → Second → Third → Resolved, без скачков
    assert!(matches!(ghost_of(&history), GhostState::Resolved { .. }));
}

// --- терминальность и ре-спавн ---

#[test]
fn resolved_ghost_is_terminal_further_ratings_change_no_ghost_state() {
    // Arrange
    let mut history = seasoned_history();
    history.ghost = Some(GhostState::Resolved {
        at: t0() - Duration::days(1),
    });
    let ghost_before = history.ghost().cloned();

    // Act: рейтинги любого знака
    rate(&mut history, Rating::Good, t0());
    rate(&mut history, Rating::Again, t0());

    // Assert: сам GhostState не изменился (счётчик копится отдельно)
    assert_eq!(history.ghost(), ghost_before.as_ref());
}

#[rstest]
#[case::after_resolved(GhostState::Resolved {
    at: Utc::now() - Duration::days(1),
})]
#[case::after_expired(GhostState::Expired {
    at: Utc::now() - Duration::days(1),
})]
fn respawn_after_terminal_state_starts_new_cycle(#[case] terminal: GhostState) {
    // Arrange: терминальное добивание, карта снова начала проваливаться
    let mut history = seasoned_history();
    history.ghost = Some(terminal);
    let now = t0();

    // Act
    rate(&mut history, Rating::Again, now);
    rate(&mut history, Rating::Again, now);

    // Assert: новый цикл — активное добивание на первой ступени
    let GhostState::Active { rung, .. } = ghost_of(&history) else {
        panic!("terminal state must be replaced by a fresh cycle");
    };
    assert_eq!(*rung, GhostRung::First);
}

#[test]
fn consecutive_again_resets_on_spawn_and_terminal_transition() {
    // Arrange: спавн из двух провалов
    let mut history = seasoned_history();
    rate(&mut history, Rating::Again, t0());
    rate(&mut history, Rating::Again, t0());
    assert_eq!(history.consecutive_again(), 0, "spawn resets the counter");

    // Act: закрываем добивание тремя успехами
    let due = ghost_due_at(&history);
    rate(&mut history, Rating::Good, due);
    let due = ghost_due_at(&history);
    rate(&mut history, Rating::Good, due);
    let due = ghost_due_at(&history);
    rate(&mut history, Rating::Good, due);

    // Assert: после резолва счётчик нулевой
    assert!(matches!(ghost_of(&history), GhostState::Resolved { .. }));
    assert_eq!(history.consecutive_again(), 0);
}

// --- TTL ---

#[test]
fn ghost_untouched_30_days_is_written_off() {
    // Arrange: последний шаг 31 день назад, окно давно открыто
    let stale = t0() - Duration::days(31);
    let mut history = seasoned_history();
    history.ghost = Some(GhostState::active(GhostRung::Second, stale, stale));

    // Act: любой явный рейтинг запускает ленивую TTL-проверку
    rate(&mut history, Rating::Good, t0());

    // Assert
    assert!(
        matches!(ghost_of(&history), GhostState::Expired { .. }),
        "ghost silent for 30+ days must be written off"
    );
    assert_eq!(history.consecutive_again(), 0);
}

#[test]
fn ghost_at_29_days_23h_stays_active() {
    // Arrange: последний шаг на минуту раньше 30-дневного порога
    let almost = t0() - Duration::days(30) + TimeDelta::minutes(1);
    let mut history = seasoned_history();
    history.ghost = Some(GhostState::active(GhostRung::Second, almost, almost));

    // Act
    rate(&mut history, Rating::Good, t0());

    // Assert: добивание живо и шаг засчитан (окно открыто)
    let GhostState::Active { rung, .. } = ghost_of(&history) else {
        panic!("ghost under TTL must stay active");
    };
    assert_eq!(*rung, GhostRung::Third);
}

#[test]
fn any_transition_resets_ttl_clock() {
    // Arrange: спавн 29 дней назад, окно открыто
    let spawned = t0() - Duration::days(29);
    let mut history = seasoned_history();
    history.ghost = Some(GhostState::active(GhostRung::First, spawned, spawned));

    // Act: успех через 29 дней (TTL ещё не истёк) продвигает ступень
    rate(&mut history, Rating::Good, t0());

    // Assert: через день после этого шага (итого 30 дней от спавна,
    // но 1 день от последнего перехода) TTL не списывает добивание
    let day_after = t0() + Duration::days(1);
    rate(&mut history, Rating::Good, day_after);
    assert!(
        history.ghost().is_some_and(GhostState::is_active),
        "TTL must count from the last transition, not from spawn"
    );
}

#[test]
fn single_again_after_ttl_expiry_does_not_respawn() {
    // Arrange: свежесписанное добивание
    let stale = t0() - Duration::days(31);
    let mut history = seasoned_history();
    history.ghost = Some(GhostState::active(GhostRung::Second, stale, stale));
    rate(&mut history, Rating::Good, t0());
    assert!(matches!(ghost_of(&history), GhostState::Expired { .. }));

    // Act: единственный провал после списания
    rate(&mut history, Rating::Again, t0());

    // Assert: счётчик был сброшен списанием — одного Again мало
    assert_eq!(history.consecutive_again(), 1);
    assert!(matches!(ghost_of(&history), GhostState::Expired { .. }));
}
