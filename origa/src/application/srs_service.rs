use crate::domain::OrigaError;
use crate::domain::{MemoryHistory, MemoryState, Rating};
use chrono::Duration;

pub struct NextReview {
    pub interval: Duration,
    pub memory_state: MemoryState,
}

pub enum RateMode {
    StandardLesson,
    FixationLesson,
}

use async_trait::async_trait;

#[async_trait]
pub trait SrsService: Send + Sync {
    async fn rate(
        &self,
        mode: RateMode,
        rating: Rating,
        memory_history: &MemoryHistory,
    ) -> Result<NextReview, OrigaError>;
}
