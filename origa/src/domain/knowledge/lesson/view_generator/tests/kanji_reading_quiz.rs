use super::*;
use crate::dictionary::kanji::KanjiInfo;
use crate::domain::knowledge::KanjiCard;
use crate::domain::knowledge::lesson::types::LessonCardView;
use crate::domain::memory::{Difficulty, MemoryState, Rating, ReviewLog, Stability};
use crate::use_cases::init_real_dictionaries;
use chrono::{Duration, Utc};
use rand::{SeedableRng, rngs::StdRng};
use std::collections::HashMap;

use super::super::LessonViewGenerator;

fn create_kanji_cards(kanji_list: &[&str]) -> Vec<Card> {
    kanji_list
        .iter()
        .map(|k| Card::Kanji(KanjiCard::new_test(k.to_string())))
        .collect()
}

fn make_kanji_knowledge_set(kanji_list: &[&str]) -> KnowledgeSet {
    let mut ks = KnowledgeSet::new();
    for k in kanji_list {
        ks.create_card(Card::Kanji(KanjiCard::new_test(k.to_string())))
            .unwrap();
    }
    ks
}

fn make_reviewed_kanji(kanji: &str, stability: f64, difficulty: f64, rating: Rating) -> StudyCard {
    let card = Card::Kanji(KanjiCard::new_test(kanji.to_string()));
    let mut sc = StudyCard::new(card);
    let mem = MemoryState::new(
        Stability::new(stability).unwrap(),
        Difficulty::new(difficulty).unwrap(),
        Utc::now(),
    );
    sc.add_review(mem, ReviewLog::new(rating, Duration::days(5)));
    sc
}

#[test]
fn generate_reading_quiz_with_sufficient_distractors() {
    init_real_dictionaries();

    let cards = create_kanji_cards(&["日", "月", "水", "火", "木"]);
    let mut kanji_cache: HashMap<String, &'static KanjiInfo> = HashMap::new();

    let result =
        generation::generate_kanji_reading_quiz(cards[0].clone(), &cards[1..], &mut kanji_cache);

    match result.expect("should succeed") {
        LessonCardView::KanjiReadingQuiz(quiz) => {
            assert_eq!(quiz.options().len(), 4);
            assert_eq!(quiz.options().iter().filter(|o| o.is_correct()).count(), 1);
        },
        other => panic!("Expected KanjiReadingQuiz, got {:?}", other),
    }
}

#[test]
fn generate_reading_quiz_fallback_insufficient_cards() {
    init_real_dictionaries();

    let cards = create_kanji_cards(&["日"]);
    let mut kanji_cache: HashMap<String, &'static KanjiInfo> = HashMap::new();

    let result = generation::generate_kanji_reading_quiz(cards[0].clone(), &[], &mut kanji_cache);

    match result.unwrap() {
        LessonCardView::Normal(c) => assert_eq!(c, cards[0]),
        other => panic!("Expected Normal fallback, got {:?}", other),
    }
}

#[test]
fn generate_reading_quiz_fallback_no_readings() {
    init_real_dictionaries();

    let card = Card::Kanji(KanjiCard::new_test("𛀀".to_string()));
    let mut kanji_cache: HashMap<String, &'static KanjiInfo> = HashMap::new();

    let result = generation::generate_kanji_reading_quiz(card.clone(), &[], &mut kanji_cache);

    match result.unwrap() {
        LessonCardView::Normal(c) => assert_eq!(c, card),
        other => panic!(
            "Expected Normal fallback for unknown kanji, got {:?}",
            other
        ),
    }
}

#[test]
fn generate_reading_quiz_filters_all_target_readings() {
    init_real_dictionaries();

    let cards = create_kanji_cards(&["日", "月", "水", "火"]);
    let mut kanji_cache: HashMap<String, &'static KanjiInfo> = HashMap::new();

    let result =
        generation::generate_kanji_reading_quiz(cards[0].clone(), &cards[1..], &mut kanji_cache);

    let info = crate::dictionary::kanji::get_kanji_info("日").unwrap();
    let target_readings: std::collections::HashSet<String> = info
        .on_readings()
        .iter()
        .chain(info.kun_readings().iter())
        .cloned()
        .collect();

    match result.expect("should succeed") {
        LessonCardView::KanjiReadingQuiz(quiz) => {
            for opt in quiz.options().iter() {
                if opt.is_correct() {
                    assert!(
                        target_readings.contains(opt.text()),
                        "correct answer '{}' must be one of target readings",
                        opt.text()
                    );
                } else {
                    assert!(
                        !target_readings.contains(opt.text()),
                        "distractor '{}' must not be any of target kanji readings",
                        opt.text()
                    );
                }
            }
        },
        other => panic!("Expected KanjiReadingQuiz, got {:?}", other),
    }
}

#[test]
fn review_kanji_produces_reading_quiz() {
    init_real_dictionaries();

    let ks = make_kanji_knowledge_set(&["日", "月", "水", "火", "木"]);
    let sc = make_reviewed_kanji("日", 5.0, 3.0, Rating::Good);
    assert!(!sc.memory().is_high_difficulty());
    let mut generator = LessonViewGenerator::new(&ks);

    let mut count = 0;
    for seed in 0..300 {
        let mut rng = StdRng::seed_from_u64(seed);
        if matches!(
            generator.apply_view(&sc, false, &mut rng),
            LessonCardView::KanjiReadingQuiz(_)
        ) {
            count += 1;
        }
    }
    assert!(count > 0, "review kanji should produce KanjiReadingQuiz");
}

#[test]
fn new_kanji_never_reading_quiz() {
    init_real_dictionaries();

    let ks = make_kanji_knowledge_set(&["日", "月", "水", "火", "木"]);
    let sc = StudyCard::new(Card::Kanji(KanjiCard::new_test("日".to_string())));
    let mut generator = LessonViewGenerator::new(&ks);

    for seed in 0..300 {
        let mut rng = StdRng::seed_from_u64(seed);
        let view = generator.apply_view(&sc, true, &mut rng);
        assert!(
            !matches!(view, LessonCardView::KanjiReadingQuiz(_)),
            "new kanji should never get KanjiReadingQuiz"
        );
    }
}

#[test]
fn high_difficulty_never_reading_quiz() {
    init_real_dictionaries();

    let ks = make_kanji_knowledge_set(&["日", "月", "水", "火", "木"]);
    let sc = make_reviewed_kanji("日", 3.0, 7.0, Rating::Hard);
    assert!(sc.memory().is_high_difficulty());
    let mut generator = LessonViewGenerator::new(&ks);

    for seed in 0..300 {
        let mut rng = StdRng::seed_from_u64(seed);
        let view = generator.apply_view(&sc, false, &mut rng);
        assert!(
            !matches!(view, LessonCardView::KanjiReadingQuiz(_)),
            "high-difficulty kanji should never get KanjiReadingQuiz"
        );
    }
}
