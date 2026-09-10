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
    /// Whether the CURRENT showing is a live AudioRecall card (view says
    /// AudioRecall AND audio is available). Sampled once per showing with
    /// untracked reads of mute/pitch-loader state, so those flips never
    /// change the mode of a card mid-answer (they apply to the next card);
    /// when `false` the card renders and behaves as `Normal` everywhere
    /// (render, keyboard, rating) — a single source of truth, no split-brain.
    pub audio_mode_active: Memo<bool>,
}

/// Builds the per-showing AudioRecall mode signal (see
/// `LessonContext::audio_mode_active`). Extracted as a free function so
/// the freeze semantics are unit-testable without mounting the lesson page.
///
/// Reactivity contract: the outer memo depends ONLY on the showing identity
/// (`showing_slot` = lesson generation + current index). Mute and
/// pitch-loader readiness are read UNTRACKED inside — flipping them never
/// recomputes the memo, so a live card keeps its mode until the user
/// advances (mode changes apply to the NEXT card, protecting the ADR-033
/// rating state machine from double-rating/stuck states).
pub fn create_audio_mode_active(
    lesson_state: RwSignal<LessonState>,
    reload_trigger: RwSignal<u32>,
    is_muted: RwSignal<bool>,
    pitch_audio_ready: RwSignal<bool>,
    native_language: RwSignal<NativeLanguage>,
) -> Memo<bool> {
    // Identity of the current showing. Recomputed on every lesson_state
    // write, but only notifies subscribers when the slot or the lesson
    // generation changes — answers/reveals of the same card keep the value.
    let showing_slot = Memo::new(move |_| {
        let state = lesson_state.get();
        (reload_trigger.get(), state.current_index)
    });

    Memo::new(move |_| {
        showing_slot.get();
        let state = lesson_state.get_untracked();
        let Some(&slot_id) = state.card_ids.get(state.current_index) else {
            return false;
        };
        let Some(lesson_card) = state.cards.get(&slot_id) else {
            return false;
        };
        if !matches!(
            lesson_card.view(),
            origa::domain::LessonCardView::AudioRecall(_)
        ) {
            return false;
        }
        let word = lesson_card
            .card()
            .question(&native_language.get_untracked())
            .map(|q| q.text().to_string())
            .unwrap_or_default();
        !is_muted.get_untracked()
            && pitch_audio_ready.get_untracked()
            && crate::ui_components::word_audio_available(&word)
    })
}
