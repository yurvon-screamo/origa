use crate::domain::OrigaError;
use crate::domain::Rating;
use crate::domain::{Difficulty, MemoryHistory, MemoryState, Stability};
use chrono::{Duration, Utc};
use rs_fsrs::{Card as FsrsCard, FSRS, Parameters, Rating as FsrsRating, State as FsrsState};
use serde::Deserialize;
use serde::Serialize;
use std::sync::OnceLock;

static FSRS_SERVICE: OnceLock<FsrsSrsService> = OnceLock::new();

struct FsrsSrsService {
    short_term_fsrs: FSRS,
    long_term_fsrs: FSRS,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NextReview {
    pub interval: Duration,
    pub memory_state: MemoryState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateMode {
    #[serde(rename = "FixationLesson")] // обратная совместимость с сериализованными данными
    ShortTerm,
    StandardLesson,
}

impl FsrsSrsService {
    fn new() -> Self {
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

        Self {
            long_term_fsrs: FSRS::new(long_term_parameters),
            short_term_fsrs: FSRS::new(short_term_parameters),
        }
    }
}

pub fn rate_memory(
    mode: RateMode,
    rating: Rating,
    memory_history: &MemoryHistory,
) -> Result<NextReview, OrigaError> {
    let srs_service = FSRS_SERVICE.get_or_init(FsrsSrsService::new);

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
        RateMode::ShortTerm => srs_service.short_term_fsrs.next(card, now, fsrs_rating),
        RateMode::StandardLesson => srs_service.long_term_fsrs.next(card, now, fsrs_rating),
    };

    let (next_review_date, interval) = if rating == Rating::Again {
        (now, Duration::zero())
    } else {
        let next_review_date = scheduling_info.card.due;
        let interval = next_review_date.signed_duration_since(now);
        let interval = if interval < Duration::zero() {
            Duration::zero()
        } else {
            interval
        };
        (next_review_date, interval)
    };

    let stability = Stability::new(scheduling_info.card.stability)?;
    let difficulty = Difficulty::new(scheduling_info.card.difficulty)?;
    let memory_state = MemoryState::new(stability, difficulty, next_review_date);

    Ok(NextReview {
        interval,
        memory_state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn rate_memory_again_returns_zero_interval_and_now_as_next_review() {
        let memory_history = MemoryHistory::new();
        let before = Utc::now();

        let result = rate_memory(RateMode::StandardLesson, Rating::Again, &memory_history).unwrap();

        let after = Utc::now();

        assert_eq!(result.interval, Duration::zero());
        let next_review = result.memory_state.next_review_date();
        assert!(*next_review >= before && *next_review <= after);
    }

    #[test]
    fn rate_memory_good_returns_future_next_review_date() {
        let memory_history = MemoryHistory::new();

        let result = rate_memory(RateMode::StandardLesson, Rating::Good, &memory_history).unwrap();

        assert!(result.interval > Duration::zero());
    }
}
