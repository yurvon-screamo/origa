use super::*;
use crate::domain::knowledge::lesson::types::{LessonCardView, YesNoCard};
use crate::domain::knowledge::KanjiCard;
use crate::domain::value_objects::NativeLanguage;
use crate::use_cases::init_real_dictionaries;
use rand::{rngs::StdRng, SeedableRng};

fn create_vocab_card_with_word(word: &str) -> Card {
    Card::Vocabulary(VocabularyCard::new(
        Question::new(word.to_string()).unwrap(),
    ))
}

fn create_yesno_card(is_correct: bool) -> YesNoCard {
    let card = create_vocab_card_with_word("テスト");
    YesNoCard::new(card, "テスト – тест".to_string(), is_correct)
}

#[test]
fn test_yesno_card_check_answer_correct_yes() {
    let yesno = create_yesno_card(true);
    assert!(yesno.check_answer(true));
}

#[test]
fn test_yesno_card_check_answer_false_no() {
    let yesno = create_yesno_card(false);
    assert!(yesno.check_answer(false));
}

#[test]
fn test_yesno_card_check_answer_wrong_yes() {
    let yesno = create_yesno_card(false);
    assert!(!yesno.check_answer(true));
}

#[test]
fn test_yesno_card_check_answer_wrong_no() {
    let yesno = create_yesno_card(true);
    assert!(!yesno.check_answer(false));
}

#[test]
fn test_generate_yesno_correct_statement() {
    init_real_dictionaries();

    let vocab_words = ["猫", "犬", "鳥", "魚"];
    let cards: Vec<Card> = vocab_words
        .iter()
        .map(|w| create_vocab_card_with_word(w))
        .collect();

    let mut rng = StdRng::seed_from_u64(42);
    let result = generation::generate_yesno(
        cards[0].clone(),
        &cards[1..],
        &NativeLanguage::Russian,
        &mut rng,
    );

    assert!(result.is_ok());
    match result.unwrap() {
        LessonCardView::YesNo(yesno) => {
            assert!(!yesno.statement_text().is_empty());
        },
        _ => panic!("Expected YesNo view"),
    }
}

#[test]
fn test_generate_yesno_false_statement() {
    init_real_dictionaries();

    let vocab_words = ["猫", "犬", "鳥", "魚"];
    let cards: Vec<Card> = vocab_words
        .iter()
        .map(|w| create_vocab_card_with_word(w))
        .collect();

    let mut rng = StdRng::seed_from_u64(123);
    let result = generation::generate_yesno(
        cards[0].clone(),
        &cards[1..],
        &NativeLanguage::Russian,
        &mut rng,
    );

    assert!(result.is_ok());
    match result.unwrap() {
        LessonCardView::YesNo(yesno) => {
            assert!(!yesno.statement_text().is_empty());
        },
        _ => panic!("Expected YesNo view"),
    }
}

#[test]
fn test_generate_yesno_fallback_no_distractors() {
    init_real_dictionaries();

    let card = create_vocab_card_with_word("猫");
    let empty_cards: Vec<Card> = vec![];

    let mut rng = StdRng::seed_from_u64(42);
    let result = generation::generate_yesno(
        card.clone(),
        &empty_cards,
        &NativeLanguage::Russian,
        &mut rng,
    );

    assert!(result.is_ok());
    match result.unwrap() {
        LessonCardView::Normal(returned_card) => {
            assert_eq!(returned_card, card);
        },
        _ => panic!("Expected Normal fallback when no distractors available"),
    }
}

#[test]
fn test_yesno_probability_distribution() {
    init_real_dictionaries();

    let vocab_words = ["猫", "犬", "鳥", "魚", "馬", "牛", "羊", "豚"];
    let cards: Vec<Card> = vocab_words
        .iter()
        .map(|w| create_vocab_card_with_word(w))
        .collect();

    let iterations = 1000;
    let mut yesno_count = 0;

    for seed in 0..iterations {
        let mut rng = StdRng::seed_from_u64(seed);
        let result = generation::generate_yesno(
            cards[0].clone(),
            &cards[1..],
            &NativeLanguage::Russian,
            &mut rng,
        );

        if let Ok(LessonCardView::YesNo(_)) = result {
            yesno_count += 1;
        }
    }

    let ratio = yesno_count as f32 / iterations as f32;
    assert!(ratio > 0.95, "YesNo generation ratio too low: {ratio}");
}

#[test]
fn test_yesno_is_correct_distribution() {
    init_real_dictionaries();

    let vocab_words = ["猫", "犬", "鳥", "魚"];
    let cards: Vec<Card> = vocab_words
        .iter()
        .map(|w| create_vocab_card_with_word(w))
        .collect();

    let iterations = 1000;
    let mut correct_count = 0;
    let mut incorrect_count = 0;

    for seed in 0..iterations {
        let mut rng = StdRng::seed_from_u64(seed);
        let result = generation::generate_yesno(
            cards[0].clone(),
            &cards[1..],
            &NativeLanguage::Russian,
            &mut rng,
        );

        if let Ok(LessonCardView::YesNo(yesno)) = result {
            if yesno.is_correct() {
                correct_count += 1;
            } else {
                incorrect_count += 1;
            }
        }
    }

    let correct_ratio = correct_count as f32 / iterations as f32;
    let incorrect_ratio = incorrect_count as f32 / iterations as f32;

    assert!(
        (0.45..=0.55).contains(&correct_ratio),
        "is_correct ratio should be ~50%, got {correct_ratio}"
    );
    assert!(
        (0.45..=0.55).contains(&incorrect_ratio),
        "is_incorrect ratio should be ~50%, got {incorrect_ratio}"
    );
}

#[test]
fn generate_yesno_kanji_with_distractors() {
    init_real_dictionaries();
    let cards: Vec<Card> = ["日", "月", "水", "火"]
        .iter()
        .map(|k| Card::Kanji(KanjiCard::new_test(k.to_string())))
        .collect();

    let mut rng = StdRng::seed_from_u64(42);
    let result = generation::generate_yesno(
        cards[0].clone(),
        &cards[1..],
        &NativeLanguage::Russian,
        &mut rng,
    );

    match result.expect("should succeed") {
        LessonCardView::YesNo(yn) => {
            assert!(!yn.statement_text().is_empty());
        },
        other => panic!("Expected YesNo for kanji, got {:?}", other),
    }
}

#[test]
fn generate_yesno_kanji_fallback_no_distractors() {
    init_real_dictionaries();
    let card = Card::Kanji(KanjiCard::new_test("日".to_string()));
    let mut rng = StdRng::seed_from_u64(42);
    let result = generation::generate_yesno(card.clone(), &[], &NativeLanguage::Russian, &mut rng);
    match result.unwrap() {
        LessonCardView::Normal(c) => assert_eq!(c, card),
        other => panic!("Expected Normal fallback, got {:?}", other),
    }
}
