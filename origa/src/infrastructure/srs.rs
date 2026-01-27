use crate::application::NextReview;
use crate::application::RateMode;
use crate::application::SrsService;
use crate::domain::OrigaError;
use crate::domain::Rating;
use crate::domain::{Difficulty, MemoryHistory, MemoryState, Stability};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use rs_fsrs::{Card as FsrsCard, FSRS, Parameters, Rating as FsrsRating, State as FsrsState};

pub struct FsrsSrsService {
    short_term_fsrs: FSRS,
    long_term_fsrs: FSRS,
}

impl FsrsSrsService {
    pub fn new() -> Result<Self, OrigaError> {
        let short_term_parameters = Parameters {
            request_retention: 0.95,
            maximum_interval: 1, // 1 day for short-term learning sessions
            enable_fuzz: true,
            enable_short_term: false,
            ..Default::default()
        };

        let long_term_parameters = Parameters {
            enable_fuzz: true,
            enable_short_term: true,
            ..Default::default()
        };

        Ok(Self {
            long_term_fsrs: FSRS::new(long_term_parameters),
            short_term_fsrs: FSRS::new(short_term_parameters),
        })
    }
}

#[async_trait(?Send)]
impl SrsService for FsrsSrsService {
    async fn rate(
        &self,
        mode: RateMode,
        rating: Rating,
        memory_history: &MemoryHistory,
    ) -> Result<NextReview, OrigaError> {
        let now = Utc::now();
        let card = if let Some(memory_state) = memory_history.memory_state() {
            let last_review_date = memory_history
                .reviews()
                .back()
                .map(|review| review.timestamp())
                .unwrap_or(now);

            let elapsed_days = now
                .signed_duration_since(last_review_date)
                .num_days()
                .max(0);

            let scheduled_days = memory_state
                .next_review_date()
                .signed_duration_since(last_review_date)
                .num_days()
                .max(0);

            let reps = memory_history.reviews().len() as i32;
            let lapses = memory_history
                .reviews()
                .iter()
                .filter(|review| matches!(review.rating(), Rating::Again))
                .count() as i32;

            FsrsCard {
                due: *memory_state.next_review_date(),
                stability: memory_state.stability().value(),
                difficulty: memory_state.difficulty().value(),
                elapsed_days,
                scheduled_days,
                reps,
                lapses,
                state: FsrsState::Review,
                last_review: last_review_date,
            }
        } else {
            FsrsCard::new()
        };

        let fsrs_rating = match rating {
            Rating::Again => FsrsRating::Again,
            Rating::Hard => FsrsRating::Hard,
            Rating::Good => FsrsRating::Good,
            Rating::Easy => FsrsRating::Easy,
        };

        let scheduling_info = match mode {
            RateMode::FixationLesson => self.short_term_fsrs.next(card, now, fsrs_rating),
            RateMode::StandardLesson => self.long_term_fsrs.next(card, now, fsrs_rating),
        };

        let next_review_date = scheduling_info.card.due;

        let interval = next_review_date.signed_duration_since(now);
        let interval = if interval < Duration::zero() || rating == Rating::Again {
            Duration::zero()
        } else {
            interval
        };

        let stability = Stability::new(scheduling_info.card.stability)?;
        let difficulty = Difficulty::new(scheduling_info.card.difficulty)?;
        let memory_state = MemoryState::new(stability, difficulty, next_review_date);

        Ok(NextReview {
            interval,
            memory_state,
        })
    }
}
