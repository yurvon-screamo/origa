use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use origa::domain::{LessonCard, MultiQuizResult, NativeLanguage, Rating};
use std::collections::{HashMap, HashSet};
use ulid::Ulid;

/// Distinguishes how a lesson was initiated.
///
/// `Normal` is the default spaced-repetition lesson driven by
/// `SelectCardsToLessonUseCase`. `GrammarPractice` is entered via the
/// grammar-detail "Practice" button under the `grammar_practice_lesson_mode`
/// feature flag; the originating grammar rule id is retained so future
/// grammar-aware card generation can consume it without an additional
/// domain round-trip.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum LessonMode {
    #[default]
    Normal,
    #[cfg(feature = "grammar_practice_lesson_mode")]
    GrammarPractice { grammar_rule_id: Ulid },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lesson_mode_default_is_normal() {
        assert_eq!(LessonMode::default(), LessonMode::Normal);
    }

    #[cfg(feature = "grammar_practice_lesson_mode")]
    #[test]
    fn grammar_practice_variant_round_trips_rule_id() {
        let id = Ulid::new();
        let mode = LessonMode::GrammarPractice {
            grammar_rule_id: id,
        };
        match mode {
            LessonMode::GrammarPractice { grammar_rule_id } => assert_eq!(grammar_rule_id, id),
            LessonMode::Normal => panic!("expected GrammarPractice variant"),
        }
    }

    #[cfg(feature = "grammar_practice_lesson_mode")]
    #[test]
    fn lesson_mode_eq_compares_rule_ids() {
        let id = Ulid::new();
        let a = LessonMode::GrammarPractice {
            grammar_rule_id: id,
        };
        let b = LessonMode::GrammarPractice {
            grammar_rule_id: id,
        };
        let c = LessonMode::GrammarPractice {
            grammar_rule_id: Ulid::new(),
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, LessonMode::Normal);
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct LessonState {
    pub mode: LessonMode,
    pub cards: HashMap<Ulid, LessonCard>,
    pub card_ids: Vec<Ulid>,
    pub current_index: usize,
    pub showing_answer: bool,
    pub review_count: usize,
    pub selected_quiz_option: Option<usize>,
    pub selected_yesno_answer: Option<bool>,
    pub dont_know_selected: bool,
    pub core_count: usize,
    pub waiting_for_next: bool,
    pub pending_rating: Option<Rating>,
    pub selected_quiz_options: HashSet<usize>,
    pub multi_quiz_submitted: bool,
    pub multi_result: Option<MultiQuizResult>,
}

#[derive(Clone)]
pub struct LessonContext {
    pub repository: HybridUserRepository,
    pub lesson_state: RwSignal<LessonState>,
    pub is_completed: RwSignal<bool>,
    pub reload_trigger: RwSignal<u32>,
    pub is_muted: RwSignal<bool>,
    pub known_kanji: RwSignal<HashSet<char>>,
    pub native_language: RwSignal<NativeLanguage>,
    pub core_count: RwSignal<usize>,
}
