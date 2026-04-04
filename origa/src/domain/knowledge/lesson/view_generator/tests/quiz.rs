use super::*;
use crate::domain::knowledge::KanjiCard;
use crate::domain::knowledge::lesson::types::LessonCardView;
use crate::domain::value_objects::NativeLanguage;
use crate::use_cases::init_real_dictionaries;
use ulid::Ulid;

mod grammar_quiz {
    use super::*;

    fn get_first_grammar_rule_id() -> Ulid {
        init_real_dictionaries();
        Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").expect("Invalid ULID")
    }

    #[test]
    fn grammar_card_generates_quiz_with_sufficient_distinct_cards() {
        init_real_dictionaries();

        let grammar_rule_id = get_first_grammar_rule_id();
        let grammar_card = create_grammar_card(grammar_rule_id);

        let rule_ids = vec![
            "01KJ9AVWBG78GHSKKD8W1YHJB3",
            "01KJ9AVWBG1AAJZXRGA499R44W",
            "01KJ9AVWBG865E0F72RYM7F34B",
        ];
        let other_cards: Vec<Card> = rule_ids
            .into_iter()
            .map(|id| create_grammar_card(Ulid::from_string(id).expect("Invalid ULID")))
            .collect();

        let lang = NativeLanguage::Russian;

        let result = generation::generate_quiz(grammar_card, &other_cards, &lang);

        assert!(result.is_ok());

        match result.unwrap() {
            LessonCardView::Quiz(quiz) => {
                assert_eq!(quiz.options().len(), 4);
                assert!(quiz.options().iter().any(|o| o.is_correct()));
            },
            _ => panic!("Expected Quiz view for grammar card with sufficient distractors"),
        }
    }

    #[test]
    fn grammar_card_returns_normal_with_insufficient_distinct_cards() {
        init_real_dictionaries();

        let grammar_rule_id = get_first_grammar_rule_id();
        let grammar_card = create_grammar_card(grammar_rule_id);

        let other_cards: Vec<Card> = vec![];

        let lang = NativeLanguage::Russian;

        let result = generation::generate_quiz(grammar_card.clone(), &other_cards, &lang);

        assert!(result.is_ok());

        match result.unwrap() {
            LessonCardView::Normal(card) => {
                assert_eq!(card, grammar_card);
            },
            _ => panic!("Expected Normal view for grammar card with insufficient distractors"),
        }
    }

    #[test]
    fn grammar_quiz_options_contain_correct_answer() {
        init_real_dictionaries();

        let grammar_rule_id = get_first_grammar_rule_id();
        let grammar_card = create_grammar_card(grammar_rule_id);

        let rule_ids = vec![
            "01KJ9AVWBG78GHSKKD8W1YHJB3",
            "01KJ9AVWBG1AAJZXRGA499R44W",
            "01KJ9AVWBG865E0F72RYM7F34B",
        ];
        let other_cards: Vec<Card> = rule_ids
            .into_iter()
            .map(|id| create_grammar_card(Ulid::from_string(id).expect("Invalid ULID")))
            .collect();

        let lang = NativeLanguage::Russian;

        let result = generation::generate_quiz(grammar_card.clone(), &other_cards, &lang);

        assert!(result.is_ok());

        match result.unwrap() {
            LessonCardView::Quiz(quiz) => {
                let correct_answer = grammar_card.answer(&lang).unwrap();
                assert!(
                    quiz.options()
                        .iter()
                        .any(|o| o.text() == correct_answer.text()),
                    "Quiz options should contain the correct answer"
                );
            },
            _ => panic!("Expected Quiz view"),
        }
    }
}

mod generate_quiz_vocab_kanji_tests {
    use super::*;

    #[test]
    fn generate_quiz_vocab_with_distinct_answers() {
        init_real_dictionaries();
        let words = ["猫", "犬", "鳥", "魚"];
        let cards: Vec<Card> = words.iter().map(|w| create_vocab_card(w)).collect();

        let result =
            generation::generate_quiz(cards[0].clone(), &cards[1..], &NativeLanguage::Russian);

        match result.expect("should succeed") {
            LessonCardView::Quiz(quiz) => {
                assert_eq!(quiz.options().len(), 4);
                assert_eq!(quiz.options().iter().filter(|o| o.is_correct()).count(), 1);
            },
            other => panic!("Expected Quiz, got {:?}", other),
        }
    }

    #[test]
    fn generate_quiz_vocab_fallback_no_cards() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");
        let result = generation::generate_quiz(card.clone(), &[], &NativeLanguage::Russian);
        match result.unwrap() {
            LessonCardView::Normal(c) => assert_eq!(c, card),
            other => panic!("Expected Normal, got {:?}", other),
        }
    }

    #[test]
    fn generate_quiz_vocab_fallback_insufficient_distinct() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");
        let same = vec![card.clone()];
        let result = generation::generate_quiz(card.clone(), &same, &NativeLanguage::Russian);
        match result.unwrap() {
            LessonCardView::Normal(c) => assert_eq!(c, card),
            other => panic!("Expected Normal, got {:?}", other),
        }
    }

    #[test]
    fn generate_quiz_kanji_with_distinct_answers() {
        init_real_dictionaries();
        let kanji_cards: Vec<Card> = ["日", "月", "水", "火"]
            .iter()
            .map(|k| Card::Kanji(KanjiCard::new_test(k.to_string())))
            .collect();

        let result = generation::generate_quiz(
            kanji_cards[0].clone(),
            &kanji_cards[1..],
            &NativeLanguage::Russian,
        );

        match result.expect("should succeed") {
            LessonCardView::Quiz(quiz) => {
                assert_eq!(quiz.options().len(), 4);
                assert_eq!(quiz.options().iter().filter(|o| o.is_correct()).count(), 1);
            },
            other => panic!("Expected Quiz for kanji, got {:?}", other),
        }
    }
}
