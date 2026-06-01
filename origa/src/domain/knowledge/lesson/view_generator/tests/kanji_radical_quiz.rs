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

fn generate_radical_quiz_for(
    kanji: &str,
    _same_type: &[&str],
) -> Result<LessonCardView, crate::domain::OrigaError> {
    let cards = create_kanji_cards(&[kanji]);
    let mut kanji_cache: HashMap<String, &'static KanjiInfo> = HashMap::new();
    super::super::kanji_radical_quiz::generate_kanji_radical_quiz(
        cards[0].clone(),
        &mut kanji_cache,
    )
}

fn extract_quiz(view: LessonCardView) -> crate::domain::knowledge::lesson::types::QuizCard {
    match view {
        LessonCardView::KanjiRadicalQuiz(quiz) => quiz,
        other => panic!("Expected KanjiRadicalQuiz, got {:?}", other),
    }
}

fn get_target_radicals(kanji: &str) -> std::collections::HashSet<char> {
    let info = crate::dictionary::kanji::get_kanji_info(kanji).unwrap();
    info.radicals_chars().iter().copied().collect()
}

#[test]
fn all_correct_radicals_are_options() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_radical_quiz_for("日", &["月", "水", "火"]).unwrap());

    let target_radicals = get_target_radicals("日");

    for &radical in &target_radicals {
        assert!(
            quiz.options()
                .iter()
                .any(|o| o.text() == radical.to_string() && o.is_correct()),
            "radical '{}' must be a correct option",
            radical
        );
    }
}

#[test]
fn no_target_radicals_in_distractors() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_radical_quiz_for("日", &["月", "水", "火"]).unwrap());
    let target_radicals = get_target_radicals("日");

    for opt in quiz.options().iter().filter(|o| !o.is_correct()) {
        let distractor_char = opt.text().chars().next().unwrap();
        assert!(
            !target_radicals.contains(&distractor_char),
            "distractor '{}' must not be a target radical",
            opt.text()
        );
    }
}

#[test]
fn mode_is_multi() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_radical_quiz_for("日", &["月", "水", "火"]).unwrap());
    assert_eq!(quiz.mode(), QuizMode::Multi);
}

#[test]
fn max_eight_options() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_radical_quiz_for("日", &["月", "水", "火"]).unwrap());
    assert!(quiz.options().len() <= 8);
}

#[test]
fn fallback_when_zero_radicals() {
    init_real_dictionaries();

    let card = Card::Kanji(KanjiCard::new_test("𛀀".to_string()));
    let mut kanji_cache: HashMap<String, &'static KanjiInfo> = HashMap::new();

    let result = super::super::kanji_radical_quiz::generate_kanji_radical_quiz(
        card.clone(),
        &mut kanji_cache,
    );

    match result.unwrap() {
        LessonCardView::Normal(c) => assert_eq!(c, card),
        other => panic!(
            "Expected Normal fallback for kanji with no radicals, got {:?}",
            other
        ),
    }
}

#[test]
fn fallback_when_radical_dictionary_not_loaded() {
    let card = Card::Kanji(KanjiCard::new_test("日".to_string()));
    let mut kanji_cache: HashMap<String, &'static KanjiInfo> = HashMap::new();

    if !crate::dictionary::radical::is_radicals_loaded() {
        let result = super::super::kanji_radical_quiz::generate_kanji_radical_quiz(
            card.clone(),
            &mut kanji_cache,
        );
        match result.unwrap() {
            LessonCardView::Normal(c) => assert_eq!(c, card),
            other => panic!(
                "Expected Normal fallback when radical dict not loaded, got {:?}",
                other
            ),
        }
    }
}

#[test]
fn check_multi_answers_perfect() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_radical_quiz_for("日", &["月", "水", "火"]).unwrap());

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

    let quiz = extract_quiz(generate_radical_quiz_for("日", &["月", "水", "火"]).unwrap());

    let correct_indices: Vec<usize> = quiz
        .options()
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_correct())
        .map(|(i, _)| i)
        .collect();

    if correct_indices.len() > 1 {
        let partial_selection = vec![correct_indices[0]];
        let result = quiz.check_multi_answers(&partial_selection);
        assert!(!result.is_perfect);
        assert!(result.correct_selections.contains(&correct_indices[0]));
        assert!(!result.missed.is_empty());
        assert!(result.wrong_selections.is_empty());
    }
}

#[test]
fn correct_radicals_have_name_as_tag() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_radical_quiz_for("日", &["月", "水", "火"]).unwrap());

    for opt in quiz.options().iter().filter(|o| o.is_correct()) {
        assert!(
            opt.tag().is_some(),
            "correct radical '{}' must have a name tag",
            opt.text()
        );
    }
}

#[test]
fn distractors_have_no_tags() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_radical_quiz_for("日", &["月", "水", "火"]).unwrap());

    for opt in quiz.options().iter().filter(|o| !o.is_correct()) {
        assert!(
            opt.tag().is_none(),
            "distractor '{}' must have no tag",
            opt.text()
        );
    }
}

#[test]
fn review_kanji_produces_radical_quiz() {
    init_real_dictionaries();

    let ks = make_kanji_knowledge_set(&["日", "月", "水", "火", "木"]);
    let sc = make_reviewed_kanji("日", 5.0, 3.0, Rating::Good);
    let mut generator = LessonViewGenerator::new(&ks);

    let mut count = 0;
    for seed in 0..300 {
        let mut rng = StdRng::seed_from_u64(seed);
        if matches!(
            generator.apply_view(&sc, false, &mut rng),
            LessonCardView::KanjiRadicalQuiz(_)
        ) {
            count += 1;
        }
    }
    assert!(count > 0, "review kanji should produce KanjiRadicalQuiz");
}

#[test]
fn new_kanji_never_radical_quiz() {
    init_real_dictionaries();

    let ks = make_kanji_knowledge_set(&["日", "月", "水", "火", "木"]);
    let sc = StudyCard::new(Card::Kanji(KanjiCard::new_test("日".to_string())));
    let mut generator = LessonViewGenerator::new(&ks);

    for seed in 0..300 {
        let mut rng = StdRng::seed_from_u64(seed);
        let view = generator.apply_view(&sc, true, &mut rng);
        assert!(
            !matches!(view, LessonCardView::KanjiRadicalQuiz(_)),
            "new kanji should never get KanjiRadicalQuiz"
        );
    }
}

#[test]
fn multi_rating_good_for_perfect() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_radical_quiz_for("日", &["月", "水", "火"]).unwrap());
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
fn multi_rating_again_for_partial() {
    init_real_dictionaries();

    let quiz = extract_quiz(generate_radical_quiz_for("日", &["月", "水", "火"]).unwrap());
    let correct_indices: Vec<usize> = quiz
        .options()
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_correct())
        .map(|(i, _)| i)
        .collect();

    if correct_indices.len() > 1 {
        let partial = vec![correct_indices[0]];
        let result = quiz.check_multi_answers(&partial);
        assert_eq!(result.rating(), Rating::Again);
    }
}
