use crate::domain::OrigaError;
use crate::domain::Rating;
use crate::domain::{CardState, Difficulty, MemoryHistory, MemoryState, Stability};
use chrono::{Duration, Utc};
use rs_fsrs::{Card as FsrsCard, FSRS, Parameters, Rating as FsrsRating, State as FsrsState};
use serde::Deserialize;
use serde::Serialize;
use std::sync::OnceLock;

static FSRS_SERVICE: OnceLock<FsrsSrsService> = OnceLock::new();

struct FsrsSrsService {
    short_term_fsrs: FSRS,
    long_term_fsrs: FSRS,
    phrase_review_fsrs: FSRS,
    grammar_fsrs: FSRS,
    kanji_fsrs: FSRS,
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
    #[serde(rename = "PhraseReview")]
    PhraseReview,
    #[serde(rename = "OnboardingScoring")]
    OnboardingScoring,
    GrammarReview,
    KanjiReview,
}

impl FsrsSrsService {
    fn new() -> Self {
        let short_term_parameters = Parameters {
            request_retention: 0.95,
            maximum_interval: 1, // 1 day for short-term learning sessions
            enable_fuzz: false,
            ..Default::default()
        };

        let long_term_parameters = Parameters {
            request_retention: 0.85,
            maximum_interval: 180,
            enable_fuzz: true,
            ..Default::default()
        };

        let phrase_review_parameters = Parameters {
            request_retention: 0.70,
            maximum_interval: 365,
            enable_fuzz: true,
            ..Default::default()
        };

        let grammar_review_parameters = Parameters {
            request_retention: 0.90,
            maximum_interval: 60,
            enable_fuzz: true,
            ..Default::default()
        };

        let kanji_review_parameters = Parameters {
            request_retention: 0.90,
            maximum_interval: 90,
            enable_fuzz: true,
            ..Default::default()
        };

        Self {
            long_term_fsrs: FSRS::new(long_term_parameters),
            short_term_fsrs: FSRS::new(short_term_parameters),
            phrase_review_fsrs: FSRS::new(phrase_review_parameters),
            grammar_fsrs: FSRS::new(grammar_review_parameters),
            kanji_fsrs: FSRS::new(kanji_review_parameters),
        }
    }
}

fn to_fsrs_state(card_state: CardState) -> FsrsState {
    match card_state {
        CardState::New => FsrsState::New,
        CardState::Learning => FsrsState::Learning,
        CardState::Review => FsrsState::Review,
        CardState::Relearning => FsrsState::Relearning,
    }
}

fn to_card_state(fsrs_state: FsrsState) -> CardState {
    match fsrs_state {
        FsrsState::New => CardState::New,
        FsrsState::Learning => CardState::Learning,
        FsrsState::Review => CardState::Review,
        FsrsState::Relearning => CardState::Relearning,
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
            state: to_fsrs_state(memory_state.card_state()),
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
        RateMode::StandardLesson | RateMode::OnboardingScoring => {
            srs_service.long_term_fsrs.next(card, now, fsrs_rating)
        },
        RateMode::PhraseReview => srs_service.phrase_review_fsrs.next(card, now, fsrs_rating),
        RateMode::GrammarReview => srs_service.grammar_fsrs.next(card, now, fsrs_rating),
        RateMode::KanjiReview => srs_service.kanji_fsrs.next(card, now, fsrs_rating),
    };

    let next_review_date = scheduling_info.card.due;
    let interval = next_review_date.signed_duration_since(now);
    let interval = if interval < Duration::zero() {
        Duration::zero()
    } else {
        interval
    };

    let stability = Stability::new(scheduling_info.card.stability)?;
    let difficulty = Difficulty::new(scheduling_info.card.difficulty)?;
    let card_state = to_card_state(scheduling_info.card.state);
    let memory_state =
        MemoryState::with_card_state(stability, difficulty, next_review_date, card_state);

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
    fn rate_memory_again_on_new_card_returns_short_interval_and_learning_state() {
        let memory_history = MemoryHistory::new();
        let before = Utc::now();

        let result = rate_memory(RateMode::StandardLesson, Rating::Again, &memory_history).unwrap();

        let after = Utc::now();

        assert!(
            result.interval >= Duration::zero() && result.interval <= Duration::minutes(1),
            "New card with Again should have interval <= 1 minute, got {:?}",
            result.interval
        );
        let next_review = result.memory_state.next_review_date();
        assert!(*next_review >= before && *next_review <= after + Duration::minutes(1));
        assert_eq!(result.memory_state.card_state(), CardState::Learning);
    }

    #[test]
    fn rate_memory_good_returns_future_next_review_date() {
        let memory_history = MemoryHistory::new();

        let result = rate_memory(RateMode::StandardLesson, Rating::Good, &memory_history).unwrap();

        assert!(result.interval > Duration::zero());
    }

    #[test]
    fn phrase_review_again_returns_short_interval() {
        let memory_history = MemoryHistory::new();

        let result = rate_memory(RateMode::PhraseReview, Rating::Again, &memory_history).unwrap();

        assert!(
            result.interval >= Duration::zero() && result.interval <= Duration::minutes(1),
            "New card with Again should have interval <= 1 minute, got {:?}",
            result.interval
        );
        assert_eq!(result.memory_state.card_state(), CardState::Learning);
    }

    #[test]
    fn phrase_review_good_returns_positive_interval() {
        let memory_history = MemoryHistory::new();

        let result = rate_memory(RateMode::PhraseReview, Rating::Good, &memory_history).unwrap();

        assert!(result.interval > Duration::zero());
    }

    #[test]
    fn phrase_review_easy_gives_longer_interval_than_standard() {
        let memory_history = MemoryHistory::new();

        let standard =
            rate_memory(RateMode::StandardLesson, Rating::Easy, &memory_history).unwrap();
        let phrase = rate_memory(RateMode::PhraseReview, Rating::Easy, &memory_history).unwrap();

        assert!(phrase.interval > standard.interval);
    }

    #[test]
    fn phrase_review_serde_roundtrip() {
        let original = RateMode::PhraseReview;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"PhraseReview\"");
        let deserialized: RateMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn onboarding_scoring_serde_roundtrip() {
        let original = RateMode::OnboardingScoring;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"OnboardingScoring\"");
        let deserialized: RateMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn grammar_review_serde_roundtrip() {
        let original = RateMode::GrammarReview;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"GrammarReview\"");
        let deserialized: RateMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn kanji_review_serde_roundtrip() {
        let original = RateMode::KanjiReview;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"KanjiReview\"");
        let deserialized: RateMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn grammar_review_again_returns_short_interval() {
        let memory_history = MemoryHistory::new();

        let result = rate_memory(RateMode::GrammarReview, Rating::Again, &memory_history).unwrap();

        assert!(
            result.interval >= Duration::zero() && result.interval <= Duration::minutes(1),
            "New card with Again should have interval <= 1 minute, got {:?}",
            result.interval
        );
    }

    #[test]
    fn kanji_review_again_returns_short_interval() {
        let memory_history = MemoryHistory::new();

        let result = rate_memory(RateMode::KanjiReview, Rating::Again, &memory_history).unwrap();

        assert!(
            result.interval >= Duration::zero() && result.interval <= Duration::minutes(1),
            "New card with Again should have interval <= 1 minute, got {:?}",
            result.interval
        );
    }

    #[test]
    fn grammar_review_good_returns_positive_interval() {
        let memory_history = MemoryHistory::new();

        let result = rate_memory(RateMode::GrammarReview, Rating::Good, &memory_history).unwrap();

        assert!(result.interval > Duration::zero());
    }

    #[test]
    fn kanji_review_good_returns_positive_interval() {
        let memory_history = MemoryHistory::new();

        let result = rate_memory(RateMode::KanjiReview, Rating::Good, &memory_history).unwrap();

        assert!(result.interval > Duration::zero());
    }

    #[test]
    fn grammar_review_easy_gives_shorter_or_equal_interval_than_standard() {
        let memory_history = MemoryHistory::new();

        let standard =
            rate_memory(RateMode::StandardLesson, Rating::Easy, &memory_history).unwrap();
        let grammar = rate_memory(RateMode::GrammarReview, Rating::Easy, &memory_history).unwrap();

        assert!(grammar.interval <= standard.interval);
    }

    #[test]
    fn kanji_review_easy_gives_shorter_or_equal_interval_than_standard() {
        let memory_history = MemoryHistory::new();

        let standard =
            rate_memory(RateMode::StandardLesson, Rating::Easy, &memory_history).unwrap();
        let kanji = rate_memory(RateMode::KanjiReview, Rating::Easy, &memory_history).unwrap();

        assert!(kanji.interval <= standard.interval);
    }

    #[test]
    fn new_card_good_transitions_to_learning() {
        let memory_history = MemoryHistory::new();
        let result = rate_memory(RateMode::StandardLesson, Rating::Good, &memory_history).unwrap();

        assert_eq!(result.memory_state.card_state(), CardState::Learning);
        assert!(
            result.interval >= Duration::zero(),
            "Learning card should have non-negative interval"
        );
    }

    #[test]
    fn new_card_easy_transitions_to_review() {
        let memory_history = MemoryHistory::new();
        let result = rate_memory(RateMode::StandardLesson, Rating::Easy, &memory_history).unwrap();

        assert_eq!(result.memory_state.card_state(), CardState::Review);
        assert!(result.interval > Duration::days(0));
    }

    #[test]
    fn review_card_again_transitions_to_relearning() {
        let mut history = MemoryHistory::new();
        let state = MemoryState::with_card_state(
            Stability::new(10.0).unwrap(),
            Difficulty::new(5.0).unwrap(),
            Utc::now() - chrono::Duration::days(5),
            CardState::Review,
        );
        history.add_review(
            state,
            crate::domain::ReviewLog::new(Rating::Good, chrono::Duration::days(5)),
        );

        let result = rate_memory(RateMode::StandardLesson, Rating::Again, &history).unwrap();

        assert_eq!(result.memory_state.card_state(), CardState::Relearning);
    }

    #[test]
    fn learning_card_good_graduates_to_review() {
        let mut history = MemoryHistory::new();
        let state = MemoryState::with_card_state(
            Stability::new(3.0).unwrap(),
            Difficulty::new(5.0).unwrap(),
            Utc::now(),
            CardState::Learning,
        );
        history.add_review(
            state,
            crate::domain::ReviewLog::new(Rating::Good, chrono::Duration::zero()),
        );

        let result = rate_memory(RateMode::StandardLesson, Rating::Good, &history).unwrap();

        assert_eq!(result.memory_state.card_state(), CardState::Review);
    }

    #[test]
    fn relearning_card_good_returns_to_review() {
        let mut history = MemoryHistory::new();
        let state = MemoryState::with_card_state(
            Stability::new(2.0).unwrap(),
            Difficulty::new(7.0).unwrap(),
            Utc::now(),
            CardState::Relearning,
        );
        history.add_review(
            state,
            crate::domain::ReviewLog::new(Rating::Again, chrono::Duration::zero()),
        );

        let result = rate_memory(RateMode::StandardLesson, Rating::Good, &history).unwrap();

        assert_eq!(result.memory_state.card_state(), CardState::Review);
    }
}
