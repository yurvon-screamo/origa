use super::*;
use crate::domain::StudyCard;
use crate::domain::knowledge::KanjiCard;
use crate::domain::knowledge::lesson::types::LessonCardView;
use crate::domain::memory::{Difficulty, MemoryState, Rating, Stability};
use crate::use_cases::init_real_dictionaries;
use chrono::Utc;
use rand::{SeedableRng, rngs::StdRng};

use super::super::LessonViewGenerator;

mod grammar_card_view_tests {
    use super::*;

    fn make_ks() -> KnowledgeSet {
        let mut ks = KnowledgeSet::new();
        for w in ["猫", "犬", "鳥"] {
            ks.create_card(Card::Vocabulary(VocabularyCard::new(
                Question::new(w.to_string()).unwrap(),
            )))
            .unwrap();
        }
        ks
    }

    #[test]
    fn grammar_new_card_always_normal() {
        let rule_id = ulid::Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").unwrap();
        let sc = StudyCard::new(Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id)));
        let ks = make_ks();
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);
        for seed in 0u64..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            assert!(matches!(
                generator.apply_view(&sc, true, &mut rng),
                LessonCardView::Normal(_)
            ));
        }
    }

    #[test]
    fn grammar_review_card_always_normal() {
        let rule_id = ulid::Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").unwrap();
        let sc = StudyCard::new(Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id)));
        let ks = make_ks();
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);
        for seed in 0u64..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            assert!(matches!(
                generator.apply_view(&sc, false, &mut rng),
                LessonCardView::Normal(_)
            ));
        }
    }
}

mod kanji_view_tests {
    use super::*;

    fn make_ks() -> KnowledgeSet {
        let mut ks = KnowledgeSet::new();
        for k in ["日", "月", "水", "火", "木"] {
            ks.create_card(Card::Kanji(KanjiCard::new_test(k.to_string())))
                .unwrap();
        }
        ks
    }

    fn make_reviewed_kanji(
        kanji: &str,
        stability: f64,
        difficulty: f64,
        rating: Rating,
    ) -> StudyCard {
        let card = Card::Kanji(KanjiCard::new_test(kanji.to_string()));
        let mut sc = StudyCard::new(card);
        let mem = MemoryState::new(
            Stability::new(stability).unwrap(),
            Difficulty::new(difficulty).unwrap(),
            Utc::now(),
        );
        sc.apply_review(mem, rating);
        sc
    }

    #[test]
    fn review_kanji_produces_yesno() {
        init_real_dictionaries();
        let ks = make_ks();
        let sc = make_reviewed_kanji("日", 5.0, 3.0, Rating::Good);
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);

        let mut yesno_count = 0;
        for seed in 0..300 {
            let mut rng = StdRng::seed_from_u64(seed);
            if matches!(
                generator.apply_view(&sc, false, &mut rng),
                LessonCardView::YesNo(_)
            ) {
                yesno_count += 1;
            }
        }
        assert!(
            yesno_count > 0,
            "review kanji (not high diff) should get YesNo sometimes"
        );
    }

    #[test]
    fn review_kanji_not_high_difficulty_produces_writing() {
        init_real_dictionaries();
        let ks = make_ks();
        let sc = make_reviewed_kanji("日", 5.0, 3.0, Rating::Good);
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);

        let mut writing_count = 0;
        for seed in 0..300 {
            let mut rng = StdRng::seed_from_u64(seed);
            if matches!(
                generator.apply_view(&sc, false, &mut rng),
                LessonCardView::Writing(_)
            ) {
                writing_count += 1;
            }
        }
        assert!(
            writing_count > 0,
            "review kanji should get Writing sometimes"
        );
    }
}

mod defensive_new_vocab_view_tests {
    use super::*;

    #[test]
    fn new_vocab_slipping_into_lesson_gets_defensive_normal() {
        init_real_dictionaries();
        let ks = create_knowledge_set_with_vocab(&["猫", "犬", "鳥", "魚"]);
        let sc = StudyCard::new(create_vocab_card("猫"));
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);

        for seed in 0..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            assert!(
                matches!(
                    generator.apply_view(&sc, true, &mut rng),
                    LessonCardView::Normal(_)
                ),
                "a new word slipping past the Exclude policy must get a defensive Normal showing"
            );
        }
    }
}

mod defensive_new_kanji_view_tests {
    use super::*;

    #[test]
    fn new_kanji_slipping_into_lesson_gets_defensive_normal() {
        init_real_dictionaries();
        let mut ks = KnowledgeSet::new();
        for k in ["日", "月", "水", "火", "木"] {
            ks.create_card(Card::Kanji(KanjiCard::new_test(k.to_string())))
                .unwrap();
        }
        let sc = StudyCard::new(Card::Kanji(KanjiCard::new_test("日".to_string())));
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);

        for seed in 0..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            assert!(
                matches!(
                    generator.apply_view(&sc, true, &mut rng),
                    LessonCardView::Normal(_)
                ),
                "a new kanji slipping past the Exclude policy must get a defensive Normal showing"
            );
        }
    }
}

/// Review-vocabulary view distribution fixtures. The late-stage fixture
/// needs a studied grammar rule applicable to the verb under test so that
/// GrammarMutated is reachable; distractor words cover QUIZ_OPTIONS_COUNT.
/// Shared by the strict-recall and repeat-pool test modules below.
const VERB_RULE_ID: &str = "01G00000000000000024000000";
const DISTRACTOR_WORDS: &[&str] = &["食べる", "飲む", "行く", "読む", "書く", "話す"];

fn make_review_vocab_ks() -> KnowledgeSet {
    init_real_dictionaries();
    let mut ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
    let rule_id = ulid::Ulid::from_string(VERB_RULE_ID).expect("valid ULID");
    let grammar_sc = ks.create_card(create_grammar_card(rule_id)).unwrap();
    // Only studied (non-new) rules participate in mutations.
    ks.mark_card_as_known(*grammar_sc.card_id()).unwrap();
    ks
}

mod review_vocab_strict_recall_tests {
    use super::*;

    const ITERATIONS: u64 = 500;

    fn make_late_stage_card(word: &str) -> StudyCard {
        create_study_card_with_memory(word, 5.0, 3.0, Rating::Good)
    }

    fn make_known_card(word: &str) -> StudyCard {
        create_study_card_with_memory(word, 25.0, 3.0, Rating::Easy)
    }

    fn make_high_difficulty_card(word: &str) -> StudyCard {
        // Four Good reviews keep the card high-difficulty (difficulty 8,
        // stability 3) while making it eligible for the Reversed format.
        let card = create_vocab_card(word);
        let mut study_card = StudyCard::new(card);
        for _ in 0..4 {
            study_card.apply_review(
                MemoryState::new(
                    Stability::new(3.0).unwrap(),
                    Difficulty::new(8.0).unwrap(),
                    Utc::now(),
                ),
                Rating::Good,
            );
        }
        assert!(study_card.memory().is_high_difficulty());
        assert_eq!(study_card.memory().good_review_count(), 4);
        study_card
    }

    fn count_views(
        study_card: &StudyCard,
        ks: &KnowledgeSet,
    ) -> std::collections::HashMap<&'static str, usize> {
        let mut generator = LessonViewGenerator::new(ks, NativeLanguage::Russian);
        let mut counts = std::collections::HashMap::<&'static str, usize>::new();
        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            let key = match generator.apply_view(study_card, false, &mut rng) {
                LessonCardView::Normal(_) => "normal",
                LessonCardView::Quiz(_) => "quiz",
                LessonCardView::YesNo(_) => "yesno",
                LessonCardView::Reversed(_) => "reversed",
                LessonCardView::GrammarMutated { .. } => "mutated",
                LessonCardView::AudioRecall(_) => "audio",
                other => panic!("unexpected view for review vocab: {other:?}"),
            };
            *counts.entry(key).or_default() += 1;
        }
        counts
    }

    #[test]
    fn in_progress_vocab_never_gets_quiz_or_yesno() {
        let ks = make_review_vocab_ks();
        let study_card = make_late_stage_card("食べる");
        assert!(study_card.memory().is_in_progress());

        let counts = count_views(&study_card, &ks);
        assert_eq!(
            counts.get("quiz").copied().unwrap_or(0),
            0,
            "late-stage (in_progress) vocab must never get a recognition Quiz"
        );
        assert_eq!(
            counts.get("yesno").copied().unwrap_or(0),
            0,
            "late-stage (in_progress) vocab must never get a YesNo card"
        );
    }

    #[test]
    fn known_vocab_never_gets_quiz_or_yesno() {
        let ks = make_review_vocab_ks();
        let study_card = make_known_card("食べる");
        assert!(study_card.memory().is_known_card());

        let counts = count_views(&study_card, &ks);
        assert_eq!(
            counts.get("quiz").copied().unwrap_or(0),
            0,
            "known vocab must never get a recognition Quiz"
        );
        assert_eq!(
            counts.get("yesno").copied().unwrap_or(0),
            0,
            "known vocab must never get a YesNo card"
        );
    }

    #[test]
    fn late_stage_vocab_produces_all_strict_recall_forms() {
        let ks = make_review_vocab_ks();
        let study_card = make_late_stage_card("食べる");

        let counts = count_views(&study_card, &ks);
        for form in ["normal", "audio", "reversed", "mutated"] {
            assert!(
                counts.get(form).copied().unwrap_or(0) > 0,
                "late-stage vocab must produce {form} views, got {counts:?}"
            );
        }
    }

    #[test]
    fn high_difficulty_vocab_produces_all_five_forms() {
        let ks = make_review_vocab_ks();
        let study_card = make_high_difficulty_card("食べる");

        let counts = count_views(&study_card, &ks);
        for form in ["normal", "quiz", "yesno", "audio", "reversed"] {
            assert!(
                counts.get(form).copied().unwrap_or(0) > 0,
                "high-difficulty vocab must produce {form} views, got {counts:?}"
            );
        }
        assert_eq!(
            counts.get("mutated").copied().unwrap_or(0),
            0,
            "high-difficulty vocab is not eligible for GrammarMutated"
        );
    }
}

mod review_vocab_repeat_pool_tests {
    use super::*;

    fn variant_name(view: &LessonCardView) -> &'static str {
        match view {
            LessonCardView::Normal(_) => "normal",
            LessonCardView::Quiz(_) => "quiz",
            LessonCardView::YesNo(_) => "yesno",
            LessonCardView::Reversed(_) => "reversed",
            LessonCardView::GrammarMutated { .. } => "mutated",
            LessonCardView::AudioRecall(_) => "audio",
            _ => "other",
        }
    }

    #[test]
    fn late_stage_repeat_pool_excludes_quiz_and_yesno() {
        let ks = make_review_vocab_ks();
        let study_card = create_study_card_with_memory("食べる", 5.0, 3.0, Rating::Good);
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);
        let mut rng = StdRng::seed_from_u64(7);

        let pool = generator.candidate_views_for_repeat(&study_card, false, &mut rng);
        let names: Vec<_> = pool.iter().map(variant_name).collect();

        assert!(
            !names.contains(&"quiz") && !names.contains(&"yesno"),
            "late-stage repeat pool must not offer recognition formats: {names:?}"
        );
        assert!(
            names.contains(&"audio"),
            "late-stage repeat pool must offer AudioRecall: {names:?}"
        );
    }

    #[test]
    fn high_difficulty_repeat_pool_includes_all_forms() {
        let ks = make_review_vocab_ks();
        let study_card = create_study_card_with_memory("食べる", 3.0, 7.0, Rating::Hard);
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);
        let mut rng = StdRng::seed_from_u64(7);

        let pool = generator.candidate_views_for_repeat(&study_card, false, &mut rng);
        let names: Vec<_> = pool.iter().map(variant_name).collect();

        for form in ["quiz", "yesno", "audio", "normal"] {
            assert!(
                names.contains(&form),
                "high-difficulty repeat pool must offer {form}: {names:?}"
            );
        }
    }
}
