use crate::domain::OrigaError;
use crate::domain::{MemoryHistory, MemoryState, Rating};
use chrono::Duration;
use std::future::Future;

pub struct NextReview {
    pub interval: Duration,
    pub memory_state: MemoryState,
}

pub enum RateMode {
    StandardLesson,
    FixationLesson,
}

pub trait SrsService {
    fn rate(
        &self,
        mode: RateMode,
        rating: Rating,
        memory_history: &MemoryHistory,
    ) -> impl Future<Output = Result<NextReview, OrigaError>>;
}
