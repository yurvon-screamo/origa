mod value;

pub use value::{Difficulty, MemoryState, Rating, ReviewLog, Stability};

use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const LOW_STABILITY_THRESHOLD: f64 = 2.0;
const KNOWN_CARD_STABILITY_THRESHOLD: f64 = 10.0;
const HIGH_DIFFICULTY_THRESHOLD: f64 = 1.75;

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

    /// Карта которая имеет низкую стабильность
    pub fn is_low_stability(&self) -> bool {
        self.stability()
            .map(|stability| stability.value() < LOW_STABILITY_THRESHOLD)
            .unwrap_or(false)
    }

    /// Карта которая имеет высокую сложность
    pub fn is_high_difficulty(&self) -> bool {
        self.difficulty()
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
        !self.is_known_card()
            && !self.is_high_difficulty()
            && !self.is_low_stability()
            && !self.is_new()
    }
}
