use super::*;
use crate::dictionary::kanji::KanjiInfo;
use crate::domain::knowledge::KanjiCard;
use crate::domain::knowledge::lesson::types::{LessonCardView, QuizMode};
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

fn get_target_readings(kanji: &str) -> std::collections::HashSet<String> {
    let info = crate::dictionary::kanji::get_kanji_info(kanji).unwrap();
    info.on_readings()
        .iter()
        .chain(info.kun_readings().iter())
        .cloned()
        .collect()
}

fn generate_reading_quiz_for(
    kanji: &str,
    same_type: &[&str],
) -> Result<LessonCardView, crate::domain::OrigaError> {
    let cards = create_kanji_cards(&[kanji]);
    let other_cards: Vec<Card> = create_kanji_cards(same_type);
    let mut kanji_cache: HashMap<String, &'static KanjiInfo> = HashMap::new();
    generation::generate_kanji_reading_quiz(cards[0].clone(), &other_cards, &mut kanji_cache)
}

fn extract_quiz(view: LessonCardView) -> crate::domain::knowledge::lesson::types::QuizCard {
    match view {
        LessonCardView::KanjiReadingQuiz(quiz) => quiz,
        other => panic!("Expected KanjiReadingQuiz, got {:?}", other),
    }
}

#[test]
fn all_target_readings_are_correct_options() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());

    let target_readings = get_target_readings("日");

    for reading in &target_readings {
        assert!(
            quiz.options()
                .iter()
                .any(|o| o.text() == reading && o.is_correct()),
            "reading '{}' must be a correct option",
            reading
        );
    }
}

#[test]
fn each_reading_has_on_or_kun_tag() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());
    let info = crate::dictionary::kanji::get_kanji_info("日").unwrap();

    for opt in quiz.options().iter().filter(|o| o.is_correct()) {
        let is_on = info.on_readings().contains(&opt.text().to_string());
        let is_kun = info.kun_readings().contains(&opt.text().to_string());
        assert!(
            is_on || is_kun,
            "correct option '{}' must be ON or KUN",
            opt.text()
        );

        if is_on {
            assert_eq!(
                opt.tag(),
                Some("ON"),
                "on-reading '{}' must have ON tag",
                opt.text()
            );
        }
        if is_kun {
            assert_eq!(
                opt.tag(),
                Some("KUN"),
                "kun-reading '{}' must have KUN tag",
                opt.text()
            );
        }
    }
}

#[test]
fn distractors_have_no_tags() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());

    for opt in quiz.options().iter().filter(|o| !o.is_correct()) {
        assert!(
            opt.tag().is_none(),
            "distractor '{}' must have no tag",
            opt.text()
        );
    }
}

#[test]
fn no_target_readings_in_distractors() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());
    let target_readings = get_target_readings("日");

    for opt in quiz.options().iter().filter(|o| !o.is_correct()) {
        assert!(
            !target_readings.contains(opt.text()),
            "distractor '{}' must not be a target reading",
            opt.text()
        );
    }
}

#[test]
fn fallback_when_more_than_six_readings() {
    init_real_dictionaries();

    let cards = create_kanji_cards(&["日"]);
    let other_cards = create_kanji_cards(&["月", "水", "火"]);
    let mut kanji_cache: HashMap<String, &'static KanjiInfo> = HashMap::new();

    let info = crate::dictionary::kanji::get_kanji_info("日").unwrap();
    let total = info.on_readings().len() + info.kun_readings().len();

    if total > 6 {
        let result = generation::generate_kanji_reading_quiz(
            cards[0].clone(),
            &other_cards,
            &mut kanji_cache,
        )
        .unwrap();
        assert!(
            matches!(result, LessonCardView::Normal(_)),
            "kanji with >6 readings must fall back to Normal"
        );
    }
}

#[test]
fn fallback_when_less_than_two_distractors() {
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
fn mode_is_multi() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());
    assert_eq!(quiz.mode(), QuizMode::Multi);
}

#[test]
fn check_multi_answers_perfect() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());

    let correct_indices: Vec<usize> = quiz
        .options()
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_correct())
        .map(|(i, _)| i)
        .collect();

    let result = quiz.check_multi_answers(&correct_indices);
    assert!(result.is_perfect);
    assert!(result.missed.is_empty());
    assert!(result.wrong_selections.is_empty());
    assert_eq!(result.correct_selections.len(), correct_indices.len());
}

#[test]
fn check_multi_answers_partial() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());

    let correct_indices: Vec<usize> = quiz
        .options()
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_correct())
        .map(|(i, _)| i)
        .collect();

    let wrong_indices: Vec<usize> = quiz
        .options()
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_correct())
        .map(|(i, _)| i)
        .collect();

    let partial_selection = vec![correct_indices[0]];
    let result = quiz.check_multi_answers(&partial_selection);
    assert!(!result.is_perfect);
    assert!(result.correct_selections.contains(&correct_indices[0]));
    assert!(!result.missed.is_empty());
    assert!(result.wrong_selections.is_empty());

    if !wrong_indices.is_empty() {
        let mixed = vec![correct_indices[0], wrong_indices[0]];
        let result2 = quiz.check_multi_answers(&mixed);
        assert!(!result2.is_perfect);
        assert!(result2.wrong_selections.contains(&wrong_indices[0]));
    }
}

#[test]
fn max_eight_options() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());
    assert!(quiz.options().len() <= 8);
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
fn generate_reading_quiz_filters_all_target_readings_from_distractors() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());
    let target_readings = get_target_readings("日");

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

#[test]
fn multi_rating_returns_good_for_perfect() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());
    let correct_indices: Vec<usize> = quiz
        .options()
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_correct())
        .map(|(i, _)| i)
        .collect();

    let result = quiz.check_multi_answers(&correct_indices);
    assert_eq!(result.rating(), Rating::Good);
}

#[test]
fn multi_rating_returns_again_for_partial() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());
    let correct_indices: Vec<usize> = quiz
        .options()
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_correct())
        .map(|(i, _)| i)
        .collect();

    let partial = vec![correct_indices[0]];
    let result = quiz.check_multi_answers(&partial);
    assert_eq!(result.rating(), Rating::Again);
}

#[test]
fn multi_rating_returns_again_for_nothing_correct() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_reading_quiz_for("日", &["月", "水", "火"]).unwrap());
    let wrong_indices: Vec<usize> = quiz
        .options()
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_correct())
        .map(|(i, _)| i)
        .collect();

    if !wrong_indices.is_empty() {
        let result = quiz.check_multi_answers(&wrong_indices);
        assert_eq!(result.rating(), Rating::Again);
    }
}
