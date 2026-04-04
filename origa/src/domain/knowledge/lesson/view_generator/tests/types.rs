use super::*;
use crate::domain::knowledge::lesson::types::{
    GrammarInfo, LessonCardView, QuizCard, QuizOption, YesNoCard,
};
use ulid::Ulid;

#[test]
fn grammar_info_new_creates_instance() {
    let info = GrammarInfo::new(None, "Title".to_string(), "Description".to_string());
    assert_eq!(info.title(), "Title");
    assert_eq!(info.description(), "Description");
}

#[test]
fn grammar_info_creates_with_rule_id() {
    let rule_id = Ulid::new();
    let info = GrammarInfo::new(
        Some(rule_id),
        "て-form".to_string(),
        "Description".to_string(),
    );
    assert_eq!(info.rule_id(), Some(rule_id));
}

#[test]
fn grammar_info_without_rule_id_returns_none() {
    let info = GrammarInfo::new(None, "て-form".to_string(), "Description".to_string());
    assert_eq!(info.rule_id(), None);
}

#[test]
fn grammar_info_returns_correct_data() {
    let info = GrammarInfo::new(
        None,
        "て-form".to_string(),
        "Форма для соединения глаголов".to_string(),
    );
    assert_eq!(info.title(), "て-form");
    assert_eq!(info.description(), "Форма для соединения глаголов");
}

#[test]
fn lesson_card_view_card_returns_inner_card() {
    let vocab = create_vocab_card("猫");

    let normal = LessonCardView::Normal(vocab.clone());
    assert_eq!(normal.card(), &vocab);

    let reversed = LessonCardView::Reversed(vocab.clone());
    assert_eq!(reversed.card(), &vocab);

    let mutated = LessonCardView::GrammarMutated {
        card: vocab.clone(),
        grammar_info: GrammarInfo::new(None, "Test".to_string(), "Test description".to_string()),
    };
    assert_eq!(mutated.card(), &vocab);

    let quiz = LessonCardView::Quiz(QuizCard::new(vocab.clone(), vec![]));
    assert_eq!(quiz.card(), &vocab);
}

mod quiz_option_and_card_tests {
    use super::*;

    #[test]
    fn quiz_option_stores_text_and_correctness() {
        let option = QuizOption::new("答え".to_string(), true);
        assert_eq!(option.text(), "答え");
        assert!(option.is_correct());
    }

    #[test]
    fn quiz_option_incorrect() {
        let option = QuizOption::new(String::new(), false);
        assert_eq!(option.text(), "");
        assert!(!option.is_correct());
    }

    #[test]
    fn quiz_card_check_answer_correct_index() {
        let options = vec![
            QuizOption::new("a".to_string(), false),
            QuizOption::new("b".to_string(), true),
            QuizOption::new("c".to_string(), false),
        ];
        let card = create_vocab_card("猫");
        let quiz = QuizCard::new(card, options);
        assert!(quiz.check_answer(1));
        assert!(!quiz.check_answer(0));
        assert!(!quiz.check_answer(2));
    }

    #[test]
    fn quiz_card_check_answer_out_of_bounds_returns_false() {
        let options = vec![QuizOption::new("only".to_string(), true)];
        let card = create_vocab_card("猫");
        let quiz = QuizCard::new(card, options);
        assert!(!quiz.check_answer(5));
        assert!(!quiz.check_answer(usize::MAX));
    }

    #[test]
    fn quiz_card_options_returns_all() {
        let card = create_vocab_card("猫");
        let opts = vec![
            QuizOption::new("a".to_string(), false),
            QuizOption::new("b".to_string(), true),
        ];
        let quiz = QuizCard::new(card, opts);
        assert_eq!(quiz.options().len(), 2);
        assert_eq!(quiz.options()[0].text(), "a");
        assert_eq!(quiz.options()[1].text(), "b");
    }

    #[test]
    fn quiz_card_card_returns_inner() {
        let card = create_vocab_card("猫");
        let quiz = QuizCard::new(card.clone(), vec![]);
        assert_eq!(quiz.card(), &card);
    }
}

mod lesson_card_view_accessors {
    use super::*;

    #[test]
    fn card_accessor_for_yesno_variant() {
        let vocab = create_vocab_card("猫");
        let yesno = YesNoCard::new(vocab.clone(), "stmt".to_string(), true);
        assert_eq!(LessonCardView::YesNo(yesno).card(), &vocab);
    }

    #[test]
    fn card_accessor_for_writing_variant() {
        let vocab = create_vocab_card("猫");
        assert_eq!(LessonCardView::Writing(vocab.clone()).card(), &vocab);
    }

    #[test]
    fn grammar_info_returns_some_for_grammar_mutated() {
        let rule_id = Ulid::new();
        let info = GrammarInfo::new(Some(rule_id), "Title".into(), "Desc".into());
        let view = LessonCardView::GrammarMutated {
            card: create_vocab_card("猫"),
            grammar_info: info,
        };
        let result = view.grammar_info().unwrap();
        assert_eq!(result.rule_id(), Some(rule_id));
        assert_eq!(result.title(), "Title");
        assert_eq!(result.description(), "Desc");
    }

    #[test]
    fn grammar_info_returns_none_for_all_other_variants() {
        let vocab = create_vocab_card("猫");
        assert!(LessonCardView::Normal(vocab.clone())
            .grammar_info()
            .is_none());
        assert!(LessonCardView::Reversed(vocab.clone())
            .grammar_info()
            .is_none());
        assert!(LessonCardView::Writing(vocab.clone())
            .grammar_info()
            .is_none());
        assert!(LessonCardView::Quiz(QuizCard::new(vocab.clone(), vec![]))
            .grammar_info()
            .is_none());
        let yesno = YesNoCard::new(vocab, "s".into(), true);
        assert!(LessonCardView::YesNo(yesno).grammar_info().is_none());
    }
}
