mod value;

pub use value::{CardState, Difficulty, MemoryState, Rating, Stability};

mod ghost;

pub use ghost::{GhostRung, GhostState};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub(crate) const KNOWN_CARD_STABILITY_THRESHOLD: f64 = 21.0;
const HIGH_DIFFICULTY_THRESHOLD: f64 = 7.0;
const HIGH_DIFFICULTY_STABILITY_CAP: f64 = 7.0;

/// Denormalized memory state for a study card.
///
/// Previously held a `VecDeque<ReviewLog>` with every individual review
/// (rating + timestamp + interval + ULID). That array was ~95% of the
/// serialized `knowledge_set` wire payload (ADR-034): 6000 cards × ~8
/// reviews × ~130 bytes ≈ 8 MB per save_sync.
///
/// Analysis of `rs-fsrs-1.2.1` proved the array feeds only two scalars
/// into the FSRS scheduler: `reps` (fuzz seed only) and `lapses` (never
/// read by scheduling math). Core FSRS state — stability, difficulty,
/// next_review_date, last_review, card_state — lives entirely in
/// `current_state`. The `easy_count`/`good_count` scalars feed only the
/// lesson view generator's reversed-card heuristic.
///
/// Cross-device merge uses `max()` for counters (known limitation: may
/// undercount by ±N during offline→offline divergence). `current_state`
/// merges via LWW by `last_review_date` (unchanged from the array-based
/// `select_later_state` logic).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryHistory {
    current_state: Option<MemoryState>,
    #[serde(default)]
    reps: u32,
    #[serde(default)]
    lapses: u32,
    #[serde(default)]
    easy_count: u32,
    #[serde(default)]
    good_count: u32,
    #[serde(default)]
    last_review_date: Option<DateTime<Utc>>,
    #[serde(default)]
    last_rating: Option<Rating>,
    /// Предспавновый счётчик подряд-провалов [consecutive_again]:
    /// два подряд `Again` на не-новой карте порождают добивание.
    /// Мержится по `max()` (принятая неточность при дивергенции).
    #[serde(default)]
    consecutive_again: u8,
    /// Добивание [GhostState] — параллельная FSRS механика закрепления.
    /// Мержится LWW по моменту последнего шага.
    #[serde(default)]
    ghost: Option<GhostState>,
    /// Отметка «знаю» [marked_known_at]: таймстемп последнего нажатия
    /// «Знаю». Карта с отметкой текущего календарного дня (UTC) молчит
    /// в канале компаньонов до конца дня. Мержится LWW по таймстемпу.
    #[serde(default)]
    marked_known_at: Option<DateTime<Utc>>,
}

impl Default for MemoryHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryHistory {
    pub fn new() -> Self {
        Self {
            current_state: None,
            reps: 0,
            lapses: 0,
            easy_count: 0,
            good_count: 0,
            last_review_date: None,
            last_rating: None,
            consecutive_again: 0,
            ghost: None,
            marked_known_at: None,
        }
    }

    pub fn ghost(&self) -> Option<&GhostState> {
        self.ghost.as_ref()
    }

    pub fn consecutive_again(&self) -> u8 {
        self.consecutive_again
    }

    pub fn marked_known_at(&self) -> Option<DateTime<Utc>> {
        self.marked_known_at
    }

    /// Отметка «знаю» в том же календарном дне, что `now`. День по UTC —
    /// тот же паттерн, что у `stats_tracker` (дневные лимиты).
    pub fn marked_known_today(&self, now: DateTime<Utc>) -> bool {
        self.marked_known_at
            .is_some_and(|ts| ts.date_naive() == now.date_naive())
    }

    /// Штампует отметку «знаю» [marked_known_at] моментом `ts`
    /// (вызывается `KnowledgeSet::mark_card_as_known`).
    pub(crate) fn set_marked_known_at(&mut self, ts: DateTime<Utc>) {
        self.marked_known_at = Some(ts);
    }

    /// Установка отметки «знаю» напрямую: time-travel («вчера/сегодня»)
    /// и сборка состояний отбора в тестах.
    #[cfg(test)]
    pub(crate) fn set_marked_known_at_for_test(&mut self, ts: Option<DateTime<Utc>>) {
        self.marked_known_at = ts;
    }

    pub fn memory_state(&self) -> Option<&MemoryState> {
        self.current_state.as_ref()
    }

    pub fn stability(&self) -> Option<&Stability> {
        self.current_state.as_ref().map(|state| state.stability())
    }

    pub fn difficulty(&self) -> Option<&Difficulty> {
        self.current_state.as_ref().map(|state| state.difficulty())
    }

    pub fn next_review_date(&self) -> Option<&DateTime<Utc>> {
        self.current_state
            .as_ref()
            .map(|state| state.next_review_date())
    }

    pub fn reps(&self) -> u32 {
        self.reps
    }

    pub fn lapses(&self) -> u32 {
        self.lapses
    }

    pub fn easy_review_count(&self) -> usize {
        self.easy_count as usize
    }

    pub fn good_review_count(&self) -> usize {
        self.good_count as usize
    }

    pub(crate) fn apply_review(&mut self, memory_state: MemoryState, rating: Rating) {
        self.current_state = Some(memory_state);
        self.last_review_date = Some(Utc::now());
        self.last_rating = Some(rating);
        self.reps += 1;
        match rating {
            Rating::Again => self.lapses += 1,
            Rating::Easy => self.easy_count += 1,
            Rating::Good => self.good_count += 1,
            Rating::Hard => {},
        }
    }

    /// Создаёт начальное состояние памяти без семантики ревью
    /// (режим знакомства, docs/acquaintance-mode.md §3): счётчики повторений,
    /// `last_review_date` и `last_rating` не трогаются — тренировка не
    /// является ревью. Единственный вызывающий — домен знакомства;
    /// наружу сырая перезапись состояния не отдаётся (см. `apply_review`).
    pub(crate) fn seed(&mut self, memory_state: MemoryState) {
        self.current_state = Some(memory_state);
    }

    pub fn last_review_date(&self) -> Option<DateTime<Utc>> {
        self.last_review_date
    }

    pub fn last_rating(&self) -> Option<Rating> {
        self.last_rating
    }

    /// Карточка которая требует повторения
    pub fn is_due(&self) -> bool {
        !self.is_new() && self.next_review_date() <= Some(&Utc::now())
    }

    /// Карточка, изучение которой еще не началось
    pub fn is_new(&self) -> bool {
        self.current_state.is_none()
    }

    /// Карточка которая имеет высокую сложность
    pub fn is_high_difficulty(&self) -> bool {
        self.difficulty()
            .map(|d| d.value() >= HIGH_DIFFICULTY_THRESHOLD)
            .unwrap_or(false)
            && self
                .stability()
                .map(|s| s.value() < HIGH_DIFFICULTY_STABILITY_CAP)
                .unwrap_or(false)
    }

    /// Карточка которая уже изучена до стабильного уровня
    pub fn is_known_card(&self) -> bool {
        self.stability()
            .map(|stability| stability.value() > KNOWN_CARD_STABILITY_THRESHOLD)
            .unwrap_or(false)
            && !self.is_high_difficulty()
    }

    /// Карточка которая еще не была изучена до стабильного уровня, но уже начала изучаться
    pub fn is_in_progress(&self) -> bool {
        !self.is_known_card() && !self.is_high_difficulty() && !self.is_new()
    }

    pub fn merge(&mut self, other: &MemoryHistory) {
        self.current_state = select_later_state(
            &self.current_state,
            &other.current_state,
            self.last_review_date,
            other.last_review_date,
        );

        // Counters merge via max(). Known limitation: during offline→offline
        // divergence this may undercount by ±N relative to a G-Set union.
        // FSRS impact: reps feeds only the fuzz seed (±5% interval jitter),
        // lapses/easy_count/good_count feed display heuristics only.
        // consecutive_again feeds the ghost spawn gate; a diverged max()
        // can at worst spawn a ghost one failed series early.
        self.reps = self.reps.max(other.reps);
        self.lapses = self.lapses.max(other.lapses);
        self.easy_count = self.easy_count.max(other.easy_count);
        self.good_count = self.good_count.max(other.good_count);
        self.consecutive_again = self.consecutive_again.max(other.consecutive_again);

        // Ghost merges LWW by the moment of its last step; terminal
        // markers (Resolved/Expired) carry their own timestamp, so a
        // resolution reached on one device is not resurrected by a stale
        // Active from another.
        self.ghost = match (self.ghost.clone(), other.ghost.clone()) {
            (Some(left), Some(right)) => {
                if right.last_transition_at() >= left.last_transition_at() {
                    Some(right)
                } else {
                    Some(left)
                }
            },
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        };

        // Отметка «знаю»: LWW по таймстемпу — позднейшее нажатие
        // побеждает (тай — правая, паттерн ghost).
        self.marked_known_at = match (self.marked_known_at, other.marked_known_at) {
            (Some(left), Some(right)) => Some(left.max(right)),
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        };

        // last_review_date + last_rating: take from whichever side is newer.
        match (self.last_review_date, other.last_review_date) {
            (Some(self_ts), Some(other_ts)) => {
                if other_ts >= self_ts {
                    self.last_review_date = other.last_review_date;
                    self.last_rating = other.last_rating;
                }
            },
            (None, Some(_)) => {
                self.last_review_date = other.last_review_date;
                self.last_rating = other.last_rating;
            },
            _ => {},
        }
    }
}

fn select_later_state(
    left: &Option<MemoryState>,
    right: &Option<MemoryState>,
    left_last_review: Option<DateTime<Utc>>,
    right_last_review: Option<DateTime<Utc>>,
) -> Option<MemoryState> {
    match (left, right) {
        (None, None) => None,
        (Some(l), None) => Some(l.clone()),
        (None, Some(r)) => Some(r.clone()),
        (Some(l), Some(r)) => match (left_last_review, right_last_review) {
            (None, None) => Some(r.clone()),
            (Some(_), None) => Some(l.clone()),
            (None, Some(_)) => Some(r.clone()),
            (Some(left_date), Some(right_date)) => {
                if right_date >= left_date {
                    Some(r.clone())
                } else {
                    Some(l.clone())
                }
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use rstest::rstest;

    fn make_state() -> MemoryState {
        MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(3.0).unwrap(),
            Utc::now(),
        )
    }

    fn make_state_at(stability: f64, difficulty: f64, date: DateTime<Utc>) -> MemoryState {
        MemoryState::new(
            Stability::new(stability).unwrap(),
            Difficulty::new(difficulty).unwrap(),
            date,
        )
    }

    // --- counter increments ---

    #[test]
    fn easy_review_count_empty_history() {
        let history = MemoryHistory::new();
        assert_eq!(history.easy_review_count(), 0);
    }

    #[test]
    fn easy_review_count_no_easy_reviews() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.apply_review(state.clone(), Rating::Good);
        history.apply_review(state.clone(), Rating::Hard);
        history.apply_review(state.clone(), Rating::Again);
        assert_eq!(history.easy_review_count(), 0);
    }

    #[test]
    fn easy_review_count_mixed_reviews() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.apply_review(state.clone(), Rating::Easy);
        history.apply_review(state.clone(), Rating::Good);
        history.apply_review(state.clone(), Rating::Easy);
        history.apply_review(state.clone(), Rating::Easy);
        history.apply_review(state.clone(), Rating::Hard);
        assert_eq!(history.easy_review_count(), 3);
    }

    #[test]
    fn easy_review_count_all_easy() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        for _ in 0..5 {
            history.apply_review(state.clone(), Rating::Easy);
        }
        assert_eq!(history.easy_review_count(), 5);
    }

    #[test]
    fn good_review_count_empty_history() {
        let history = MemoryHistory::new();
        assert_eq!(history.good_review_count(), 0);
    }

    #[test]
    fn good_review_count_no_good_reviews() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.apply_review(state.clone(), Rating::Easy);
        history.apply_review(state.clone(), Rating::Hard);
        assert_eq!(history.good_review_count(), 0);
    }

    #[test]
    fn good_review_count_mixed_reviews() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.apply_review(state.clone(), Rating::Good);
        history.apply_review(state.clone(), Rating::Easy);
        history.apply_review(state.clone(), Rating::Good);
        history.apply_review(state.clone(), Rating::Hard);
        history.apply_review(state.clone(), Rating::Good);
        assert_eq!(history.good_review_count(), 3);
    }

    #[test]
    fn reps_increments_on_every_rating() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.apply_review(state.clone(), Rating::Good);
        history.apply_review(state.clone(), Rating::Easy);
        history.apply_review(state.clone(), Rating::Hard);
        history.apply_review(state.clone(), Rating::Again);
        assert_eq!(history.reps(), 4);
    }

    #[test]
    fn lapses_increments_only_on_again() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.apply_review(state.clone(), Rating::Good);
        history.apply_review(state.clone(), Rating::Again);
        history.apply_review(state.clone(), Rating::Easy);
        history.apply_review(state.clone(), Rating::Again);
        assert_eq!(history.lapses(), 2);
    }

    #[test]
    fn last_rating_tracks_most_recent() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.apply_review(state.clone(), Rating::Good);
        assert_eq!(history.last_rating(), Some(Rating::Good));
        history.apply_review(make_state(), Rating::Hard);
        assert_eq!(history.last_rating(), Some(Rating::Hard));
    }

    // --- is_due ---

    #[test]
    fn is_due_true_when_next_review_in_past() {
        let past = Utc::now() - Duration::days(1);
        let state = MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(3.0).unwrap(),
            past,
        );
        let mut history = MemoryHistory::new();
        history.apply_review(state.clone(), Rating::Good);

        assert!(history.is_due());
    }

    #[test]
    fn is_due_false_when_next_review_in_future() {
        let future = Utc::now() + Duration::days(1);
        let state = MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(3.0).unwrap(),
            future,
        );
        let mut history = MemoryHistory::new();
        history.apply_review(state.clone(), Rating::Good);

        assert!(!history.is_due());
    }

    #[test]
    fn is_due_false_when_no_memory_state() {
        let history = MemoryHistory::new();

        assert!(!history.is_due());
    }

    // --- is_high_difficulty ---

    #[test]
    fn is_high_difficulty_true_above_threshold() {
        let state = MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(HIGH_DIFFICULTY_THRESHOLD + 0.1).unwrap(),
            Utc::now(),
        );
        let mut history = MemoryHistory::new();
        history.apply_review(state.clone(), Rating::Hard);

        assert!(history.is_high_difficulty());
    }

    #[test]
    fn is_high_difficulty_false_below_threshold() {
        let state = MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(HIGH_DIFFICULTY_THRESHOLD - 0.1).unwrap(),
            Utc::now(),
        );
        let mut history = MemoryHistory::new();
        history.apply_review(state.clone(), Rating::Good);

        assert!(!history.is_high_difficulty());
    }

    #[test]
    fn is_high_difficulty_false_when_no_memory_state() {
        let history = MemoryHistory::new();

        assert!(!history.is_high_difficulty());
    }

    #[test]
    fn is_high_difficulty_false_when_stability_above_cap() {
        let state = MemoryState::new(
            Stability::new(HIGH_DIFFICULTY_STABILITY_CAP + 1.0).unwrap(),
            Difficulty::new(HIGH_DIFFICULTY_THRESHOLD + 0.1).unwrap(),
            Utc::now(),
        );
        let mut history = MemoryHistory::new();
        history.apply_review(state.clone(), Rating::Hard);

        assert!(!history.is_high_difficulty());
    }

    #[test]
    fn is_high_difficulty_true_when_stability_below_cap() {
        let state = MemoryState::new(
            Stability::new(HIGH_DIFFICULTY_STABILITY_CAP - 1.0).unwrap(),
            Difficulty::new(HIGH_DIFFICULTY_THRESHOLD + 0.1).unwrap(),
            Utc::now(),
        );
        let mut history = MemoryHistory::new();
        history.apply_review(state.clone(), Rating::Hard);

        assert!(history.is_high_difficulty());
    }

    // --- is_in_progress ---

    #[test]
    fn is_in_progress_true_when_stability_below_threshold() {
        let state = MemoryState::new(
            Stability::new(KNOWN_CARD_STABILITY_THRESHOLD - 0.1).unwrap(),
            Difficulty::new(3.0).unwrap(),
            Utc::now() + Duration::days(1),
        );
        let mut history = MemoryHistory::new();
        history.apply_review(state.clone(), Rating::Good);

        assert!(history.is_in_progress());
    }

    #[test]
    fn is_in_progress_false_when_stability_above_threshold() {
        let state = MemoryState::new(
            Stability::new(KNOWN_CARD_STABILITY_THRESHOLD + 0.1).unwrap(),
            Difficulty::new(3.0).unwrap(),
            Utc::now() + Duration::days(1),
        );
        let mut history = MemoryHistory::new();
        history.apply_review(state.clone(), Rating::Good);

        assert!(!history.is_in_progress());
    }

    #[test]
    fn is_in_progress_false_when_no_memory_state() {
        let history = MemoryHistory::new();

        assert!(!history.is_in_progress());
    }

    // --- merge ---

    #[test]
    fn merge_empty_with_non_empty_result_is_non_empty() {
        let mut empty = MemoryHistory::new();
        let state = make_state();
        let mut non_empty = MemoryHistory::new();
        non_empty.apply_review(state.clone(), Rating::Good);

        empty.merge(&non_empty);

        assert!(empty.memory_state().is_some());
        assert_eq!(empty.reps(), 1);
    }

    #[test]
    fn merge_non_empty_with_empty_result_is_non_empty() {
        let state = make_state();
        let mut non_empty = MemoryHistory::new();
        non_empty.apply_review(state.clone(), Rating::Good);
        let original_state = non_empty.memory_state().cloned();
        let empty = MemoryHistory::new();

        non_empty.merge(&empty);

        assert_eq!(non_empty.memory_state(), original_state.as_ref());
        assert_eq!(non_empty.reps(), 1);
    }

    #[test]
    fn merge_combines_counters_via_max() {
        let state1 = make_state_at(3.0, 2.0, Utc::now() - Duration::days(2));
        let state2 = make_state_at(5.0, 4.0, Utc::now());

        let mut history_a = MemoryHistory::new();
        history_a.apply_review(state1, Rating::Good);
        history_a.apply_review(make_state(), Rating::Easy);
        history_a.apply_review(make_state(), Rating::Again);

        let mut history_b = MemoryHistory::new();
        history_b.apply_review(state2, Rating::Easy);

        history_a.merge(&history_b);

        // reps: max(3, 1) = 3
        assert_eq!(history_a.reps(), 3);
        // easy_count: max(1, 1) = 1
        assert_eq!(history_a.easy_review_count(), 1);
        // lapses: max(1, 0) = 1
        assert_eq!(history_a.lapses(), 1);
        // select_later_state picks state2 (newer last_review_date)
        assert_eq!(history_a.memory_state().unwrap().difficulty().value(), 4.0);
    }

    #[test]
    fn merge_takes_last_rating_from_newer_side() {
        let older = make_state_at(3.0, 2.0, Utc::now() - Duration::days(1));
        let newer = make_state_at(5.0, 4.0, Utc::now());

        let mut history_a = MemoryHistory::new();
        history_a.apply_review(older, Rating::Good);

        let mut history_b = MemoryHistory::new();
        history_b.apply_review(newer, Rating::Easy);

        history_a.merge(&history_b);

        assert_eq!(history_a.last_rating(), Some(Rating::Easy));
    }

    // --- добивания: независимость FSRS, мерж, back-compat ---

    #[test]
    fn rating_moves_fsrs_and_ghost_independently() {
        // Arrange: apply_review (FSRS-путь) не трогает добивание
        let mut history = MemoryHistory::new();
        history.apply_review(make_state(), Rating::Again);
        history.apply_review(make_state(), Rating::Again);

        // Assert: состояние памяти обновилось, добивания нет
        assert!(history.memory_state().is_some());
        assert_eq!(history.ghost(), None);
        assert_eq!(history.consecutive_again(), 0);
    }

    #[test]
    fn seed_from_acquaintance_spawns_no_ghost() {
        // Arrange: сидирование знакомства не является ревью
        let mut history = MemoryHistory::new();
        history.seed(make_state());

        // Assert: состояние появилось, счётчики и добивание нетронуты
        assert!(history.memory_state().is_some());
        assert_eq!(history.ghost(), None);
        assert_eq!(history.consecutive_again(), 0);
    }

    #[test]
    fn merge_takes_ghost_counters_via_max_and_state_via_lww() {
        // Arrange: дивергенция — разные счётчики, разные шаги лестницы
        let now = Utc::now();
        let mut left = MemoryHistory::new();
        left.consecutive_again = 3;
        left.ghost = Some(GhostState::Active {
            rung: GhostRung::First,
            due_at: now,
            last_transition_at: now - Duration::days(2),
        });

        let mut right = MemoryHistory::new();
        right.consecutive_again = 1;
        right.ghost = Some(GhostState::Active {
            rung: GhostRung::Third,
            due_at: now,
            last_transition_at: now,
        });

        // Act
        left.merge(&right);

        // Assert: счётчик по max, состояние по более позднему шагу
        assert_eq!(left.consecutive_again(), 3);
        let GhostState::Active { rung, .. } = left.ghost().unwrap() else {
            panic!("ghost must stay active");
        };
        assert_eq!(*rung, GhostRung::Third);
    }

    #[test]
    fn merge_preserves_ghost_when_other_side_has_none() {
        // Arrange: одна сторона не знала о добивании
        let now = Utc::now();
        let mut left = MemoryHistory::new();
        left.ghost = Some(GhostState::Resolved { at: now });
        let right = MemoryHistory::new();

        // Act
        left.merge(&right);

        // Assert
        assert!(matches!(left.ghost(), Some(GhostState::Resolved { .. })));
    }

    #[test]
    fn merge_terminal_marker_wins_over_stale_active() {
        // Arrange: устройство A закрыло добивание позже, чем B сделало шаг
        let now = Utc::now();
        let mut resolved_side = MemoryHistory::new();
        resolved_side.ghost = Some(GhostState::Resolved {
            at: now - Duration::days(1),
        });
        let mut stale_active_side = MemoryHistory::new();
        stale_active_side.ghost = Some(GhostState::Active {
            rung: GhostRung::Second,
            due_at: now - Duration::days(4),
            last_transition_at: now - Duration::days(3),
        });

        // Act
        stale_active_side.merge(&resolved_side);

        // Assert: резолв не воскрес старым Active
        assert!(matches!(
            stale_active_side.ghost(),
            Some(GhostState::Resolved { .. })
        ));
    }

    #[test]
    fn legacy_serialized_history_deserializes_with_default_ghost_fields() {
        // Arrange: формат данных до добиваний — без ghost/consecutive_again
        let json = r#"{
            "current_state": null,
            "reps": 2,
            "lapses": 1
        }"#;

        // Act
        let history: MemoryHistory = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(history.reps(), 2);
        assert_eq!(history.lapses(), 1);
        assert_eq!(history.ghost(), None);
        assert_eq!(history.consecutive_again(), 0);
        assert_eq!(history.marked_known_at(), None);
    }

    // --- Отметка «знаю» [marked_known_at] ---

    fn today_midnight_from(now: DateTime<Utc>) -> DateTime<Utc> {
        use chrono::{Datelike, TimeZone};
        let date = now.date_naive();
        Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
            .unwrap()
    }

    /// Кейсы-константы: штампы строятся в теле от единого `now`, чтобы
    /// тест не зависел от перехода UTC-полуночи между атрибутом и телом.
    #[derive(Clone, Copy)]
    enum StampCase {
        Today,
        MidnightToday,
        Yesterday,
        None,
    }

    #[rstest]
    #[case::same_day_stamp(StampCase::Today, true)]
    #[case::midnight_edge_same_day(StampCase::MidnightToday, true)]
    #[case::previous_day_stamp(StampCase::Yesterday, false)]
    #[case::no_stamp(StampCase::None, false)]
    fn marked_known_today_matches_only_same_utc_day(
        #[case] case: StampCase,
        #[case] expected: bool,
    ) {
        // Arrange
        let now = Utc::now();
        let stamp = match case {
            StampCase::Today => Some(now),
            StampCase::MidnightToday => Some(today_midnight_from(now)),
            StampCase::Yesterday => Some(now - Duration::hours(25)),
            StampCase::None => None,
        };
        let mut history = MemoryHistory::new();
        history.set_marked_known_at_for_test(stamp);

        // Act
        let is_today = history.marked_known_today(now);

        // Assert
        assert_eq!(is_today, expected);
    }

    #[test]
    fn merge_marked_known_at_takes_later_timestamp() {
        // Arrange
        let later = Utc::now();
        let earlier = later - Duration::hours(2);
        let mut left = MemoryHistory::new();
        left.set_marked_known_at_for_test(Some(earlier));
        let mut right = MemoryHistory::new();
        right.set_marked_known_at_for_test(Some(later));

        // Act
        left.merge(&right);

        // Assert
        assert_eq!(left.marked_known_at(), Some(later));
    }

    #[test]
    fn merge_marked_known_at_propagates_from_other_side() {
        // Arrange
        let stamp = Utc::now() - Duration::hours(1);
        let mut left = MemoryHistory::new();
        let mut right = MemoryHistory::new();
        right.set_marked_known_at_for_test(Some(stamp));

        // Act
        left.merge(&right);

        // Assert
        assert_eq!(left.marked_known_at(), Some(stamp));
    }
}
