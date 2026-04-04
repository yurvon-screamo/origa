use super::*;
use crate::domain::knowledge::lesson::types::LessonCardView;
use crate::domain::memory::Rating;
use crate::use_cases::init_real_dictionaries;
use rand::{rngs::StdRng, SeedableRng};

use super::super::LessonViewGenerator;

mod yesno_view_filtering {
    use super::*;

    fn create_high_difficulty_card(word: &str) -> StudyCard {
        let study_card = create_study_card_with_memory(word, 3.0, 7.0, 5, Rating::Hard);
        assert!(study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_known_card());
        assert!(!study_card.memory().is_in_progress());
        study_card
    }

    fn create_in_progress_card(word: &str) -> StudyCard {
        let study_card = create_study_card_with_memory(word, 5.0, 3.0, 5, Rating::Good);
        assert!(study_card.memory().is_in_progress());
        assert!(!study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_known_card());
        study_card
    }

    const DISTRACTOR_WORDS: &[&str] = &["猫", "犬", "鳥", "魚", "馬", "牛"];
    const ITERATIONS: u64 = 500;

    fn count_yesno_views(study_card: &StudyCard, ks: &KnowledgeSet) -> (usize, usize) {
        let generator = LessonViewGenerator::new(ks);
        let mut yesno_count = 0;
        let mut other_count = 0;

        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rng);

            match view {
                LessonCardView::YesNo(_) => yesno_count += 1,
                _ => other_count += 1,
            }
        }

        (yesno_count, other_count)
    }

    #[test]
    fn high_difficulty_card_never_gets_yesno_view() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_high_difficulty_card("猫");

        let (yesno, _other) = count_yesno_views(&study_card, &ks);

        assert_eq!(
            yesno, 0,
            "high_difficulty card should never get YesNo view, got {yesno} YesNo out of {ITERATIONS} iterations"
        );
    }

    #[test]
    fn in_progress_card_can_get_yesno_view() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_in_progress_card("猫");

        let (yesno, _other) = count_yesno_views(&study_card, &ks);

        assert!(
            yesno > 0,
            "in_progress card should be able to get YesNo view, got 0 YesNo out of {ITERATIONS} iterations"
        );
    }
}

mod reversed_view_filtering {
    use super::*;

    fn create_high_difficulty_card(word: &str) -> StudyCard {
        let study_card = create_study_card_with_memory(word, 3.0, 7.0, 5, Rating::Hard);
        assert!(study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_known_card());
        assert!(!study_card.memory().is_in_progress());
        study_card
    }

    fn create_known_card(word: &str) -> StudyCard {
        let study_card = create_study_card_with_memory(word, 15.0, 2.0, 20, Rating::Easy);
        assert!(study_card.memory().is_known_card());
        assert!(!study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_in_progress());
        study_card
    }

    fn create_in_progress_card(word: &str) -> StudyCard {
        let study_card = create_study_card_with_memory(word, 5.0, 3.0, 5, Rating::Good);
        assert!(study_card.memory().is_in_progress());
        assert!(!study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_known_card());
        study_card
    }

    const DISTRACTOR_WORDS: &[&str] = &["猫", "犬", "鳥", "魚", "馬", "牛"];
    const ITERATIONS: u64 = 500;

    fn count_view_types(study_card: &StudyCard, ks: &KnowledgeSet) -> (usize, usize, usize) {
        let generator = LessonViewGenerator::new(ks);
        let mut reversed_count = 0;
        let mut grammar_mutated_count = 0;
        let mut normal_count = 0;

        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rng);

            match view {
                LessonCardView::Reversed(_) => reversed_count += 1,
                LessonCardView::GrammarMutated { .. } => grammar_mutated_count += 1,
                LessonCardView::Normal(_) => normal_count += 1,
                _ => {},
            }
        }

        (reversed_count, grammar_mutated_count, normal_count)
    }

    #[test]
    fn high_difficulty_card_never_gets_reversed_view() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_high_difficulty_card("猫");

        let (reversed, grammar_mutated, normal) = count_view_types(&study_card, &ks);

        assert_eq!(
            reversed, 0,
            "high_difficulty card should never get Reversed view"
        );
        assert_eq!(
            grammar_mutated, 0,
            "high_difficulty card should never get GrammarMutated view"
        );
        assert!(
            normal > 0,
            "high_difficulty card should get Normal view as fallback in advanced range"
        );
    }

    #[test]
    fn known_card_can_get_reversed_view() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_known_card("猫");

        let (reversed, grammar_mutated, normal) = count_view_types(&study_card, &ks);

        assert!(
            reversed > 0 || grammar_mutated > 0,
            "known card should get Reversed or GrammarMutated views, got {reversed} reversed, {grammar_mutated} grammar_mutated, {normal} normal"
        );
    }

    #[test]
    fn in_progress_card_can_get_reversed_view() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_in_progress_card("猫");

        let (reversed, grammar_mutated, normal) = count_view_types(&study_card, &ks);

        assert!(
            reversed > 0 || grammar_mutated > 0,
            "in_progress card should get Reversed or GrammarMutated views, got {reversed} reversed, {grammar_mutated} grammar_mutated, {normal} normal"
        );
    }
}
