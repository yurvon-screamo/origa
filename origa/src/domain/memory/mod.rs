mod value;

pub use value::{Difficulty, MemoryState, Rating, ReviewLog, Stability};

use std::collections::{HashSet, VecDeque};

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

const KNOWN_CARD_STABILITY_THRESHOLD: f64 = 10.0;
const HIGH_DIFFICULTY_THRESHOLD: f64 = 5.0;
const MAX_DAYS_INTERVAL_THRESHOLD: i64 = 10;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MemoryHistory {
    current_state: Option<MemoryState>,
    reviews: VecDeque<ReviewLog>,
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
            reviews: VecDeque::new(),
        }
    }

    pub fn memory_state(&self) -> Option<&MemoryState> {
        self.current_state.as_ref()
    }

    pub fn latest_interval(&self) -> TimeDelta {
        self.reviews
            .back()
            .map(|x| x.interval())
            .unwrap_or(TimeDelta::zero())
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

    pub fn reviews(&self) -> &VecDeque<ReviewLog> {
        &self.reviews
    }

    pub(crate) fn add_review(&mut self, memory_state: MemoryState, review: ReviewLog) {
        self.current_state = Some(memory_state);
        self.reviews.push_back(review);
    }

    pub fn last_review_date(&self) -> Option<DateTime<Utc>> {
        self.reviews.back().map(|review| review.timestamp())
    }

    /// Карта которая требует повторения
    pub fn is_due(&self) -> bool {
        !self.is_new() && self.next_review_date() <= Some(&Utc::now())
    }

    /// Карта изучение которой еще не началось``
    pub fn is_new(&self) -> bool {
        self.current_state.is_none()
    }

    /// Карта которая имеет высокую сложность
    pub fn is_high_difficulty(&self) -> bool {
        self.latest_interval().num_days() <= MAX_DAYS_INTERVAL_THRESHOLD
            && self
                .difficulty()
                .map(|difficulty| difficulty.value() >= HIGH_DIFFICULTY_THRESHOLD)
                .unwrap_or(false)
    }

    /// Карта которая уже изучена до стабильного уровня
    pub fn is_known_card(&self) -> bool {
        self.stability()
            .map(|stability| stability.value() > KNOWN_CARD_STABILITY_THRESHOLD)
            .unwrap_or(false)
            && !self.is_high_difficulty()
    }

    /// Карта которая еще не была изучена до стабильного уровня, но уже начала изучаться
    pub fn is_in_progress(&self) -> bool {
        !self.is_known_card() && !self.is_high_difficulty() && !self.is_new()
    }

    pub fn merge(&mut self, other: &MemoryHistory) {
        self.current_state = select_later_state(
            &self.current_state,
            &other.current_state,
            self.last_review_date(),
            other.last_review_date(),
        );

        let existing_ids: HashSet<Ulid> = self.reviews.iter().map(|r| r.id()).collect();

        for review in &other.reviews {
            if !existing_ids.contains(&review.id()) {
                self.reviews.push_back(*review);
            }
        }

        self.reviews
            .make_contiguous()
            .sort_by_key(|r| r.timestamp());
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
            }
        },
    }
}
