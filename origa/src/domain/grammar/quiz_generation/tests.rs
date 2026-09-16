//! Behavior spec for grammar quiz generation (#503).
//!
//! The near-miss contract: distractors keep the rule's visible tail
//! (postfix chains) or conjugation ending (pure chains) and break the
//! stem instead — never postfixes borrowed from unrelated rules.

use super::*;

#[test]
fn generate_distractors_excludes_correct() {
    let actions = vec![FormatAction::VerbToMasu {}];
    let distractors =
        generate_grammar_distractors(&actions, "行く", &PartOfSpeech::Verb, "行きます", 3);
    assert!(distractors.iter().all(|d| d != "行きます"));
}

#[test]
fn generate_distractors_no_duplicates() {
    let actions = vec![FormatAction::VerbToMasu {}];
    let distractors =
        generate_grammar_distractors(&actions, "行く", &PartOfSpeech::Verb, "行きます", 3);
    let unique: HashSet<_> = distractors.iter().collect();
    assert_eq!(unique.len(), distractors.len());
}

#[test]
fn generate_distractors_returns_up_to_count() {
    let actions = vec![FormatAction::VerbToMasu {}];
    let distractors =
        generate_grammar_distractors(&actions, "行く", &PartOfSpeech::Verb, "行きます", 3);
    assert!(distractors.len() <= 3);
}

#[test]
fn find_known_vocab_returns_empty_for_empty_set() {
    let ks = KnowledgeSet::new();
    let result = find_known_vocab_words_for_pos(&ks, &PartOfSpeech::Verb);
    assert!(result.is_empty());
}

/// Near-miss distractor contract (#503): a quiz option set is solvable by
/// pattern spotting when the rule's own visible tail (postfix / ending)
/// appears in exactly one option. Distractors must therefore keep the
/// rule's tail intact and mutate the stem instead — the mistakes a
/// learner actually makes (wrong vowel row, forgot-to-conjugate, small
/// っ, voicing), never postfixes borrowed from unrelated rules.
mod near_miss_distractors {
    use super::*;
    use crate::dictionary::grammar::FormatAction;

    fn distractors_for(
        actions: Vec<FormatAction>,
        word: &str,
        pos: PartOfSpeech,
        correct: &str,
    ) -> Vec<String> {
        let mut result = generate_grammar_distractors(&actions, word, &pos, correct, 3);
        result.sort();
        result
    }

    #[test]
    fn postfix_only_chain_keeps_rule_tail() {
        // Arrange — ～つもりです: [AddPostfix] on the dictionary form.
        // Old behavior: distractors borrowed foreign postfixes (書くこと,
        // 書くはず, …) so exactly one option contained つもりです.
        let actions = vec![FormatAction::AddPostfix {
            postfix: "つもりです".into(),
        }];

        // Act
        let distractors = distractors_for(actions, "書く", PartOfSpeech::Verb, "書くつもりです");

        // Assert — every option still carries the rule's own tail.
        assert_eq!(distractors.len(), 3);
        assert!(
            distractors.iter().all(|d| d.ends_with("つもりです")),
            "distractors must keep the rule tail, got {distractors:?}"
        );
        assert!(
            !distractors.iter().any(|d| d == "書くつもりです"),
            "distractors must not repeat the correct answer"
        );
    }

    #[test]
    fn pure_masu_chain_breaks_the_stem_not_the_ending() {
        // Arrange — ～ます: [VerbToMasu]; correct 書きます.
        // Old behavior: random same-group actions (書ける, 書かせる, …) —
        // valid forms of OTHER rules, ending differs from ます.
        let actions = vec![FormatAction::VerbToMasu {}];

        // Act
        let distractors = distractors_for(actions, "書く", PartOfSpeech::Verb, "書きます");

        // Assert — near-misses share the ます ending, stem is broken:
        // forgot-to-conjugate (書くます), wrong vowel row (書かます/書けます).
        assert_eq!(distractors.len(), 3);
        assert!(
            distractors.iter().all(|d| d.ends_with("ます")),
            "distractors must keep the ます ending, got {distractors:?}"
        );
        assert!(
            distractors
                .iter()
                .all(|d| *d != "書ける" && *d != "書かせる"),
            "distractors must not be valid forms of other rules"
        );
    }

    #[test]
    fn pure_nai_chain_contains_forgot_to_conjugate() {
        // Arrange — ～ない: [VerbToNai]; correct 書かない. The classic
        // learner mistake is keeping the dictionary form: 書くない.
        let actions = vec![FormatAction::VerbToNai {}];

        // Act
        let distractors = distractors_for(actions, "書く", PartOfSpeech::Verb, "書かない");

        // Assert — the forgot-to-conjugate error must be offered.
        assert!(distractors.contains(&"書くない".to_string()));
        assert!(
            distractors.iter().all(|d| d.ends_with("ない")),
            "distractors must keep the ない ending, got {distractors:?}"
        );
    }

    #[test]
    fn conjugated_postfix_chain_keeps_rule_tail() {
        // Arrange — ～てください: [VerbToTeForm, AddPostfix ください].
        let actions = vec![
            FormatAction::VerbToTeForm {},
            FormatAction::AddPostfix {
                postfix: "ください".into(),
            },
        ];

        // Act
        let distractors = distractors_for(actions, "書く", PartOfSpeech::Verb, "書いてください");

        // Assert — tail preserved while the te-base is replaced by other
        // full forms of the word (ta / masu / nai …).
        assert_eq!(distractors.len(), 3);
        assert!(
            distractors.iter().all(|d| d.ends_with("ください")),
            "distractors must keep the rule tail, got {distractors:?}"
        );
    }

    #[test]
    fn i_adjective_postfix_chain_keeps_rule_tail() {
        // Arrange — ～くなります: [AdjectiveToKu, AddPostfix なります].
        let actions = vec![
            FormatAction::AdjectiveToKu {},
            FormatAction::AddPostfix {
                postfix: "なります".into(),
            },
        ];

        // Act
        let distractors =
            distractors_for(actions, "高い", PartOfSpeech::IAdjective, "高くなります");

        // Assert
        assert_eq!(distractors.len(), 3);
        assert!(
            distractors.iter().all(|d| d.ends_with("なります")),
            "distractors must keep the rule tail, got {distractors:?}"
        );
    }

    #[test]
    fn na_adjective_postfix_chain_keeps_rule_tail() {
        // Arrange — ～です on a na-adjective: [AddPostfix です].
        let actions = vec![FormatAction::AddPostfix {
            postfix: "です".into(),
        }];

        // Act
        let distractors = distractors_for(actions, "静か", PartOfSpeech::NaAdjective, "静かです");

        // Assert — 静かなです is the textbook mistake for this rule.
        assert_eq!(distractors.len(), 3);
        assert!(
            distractors.iter().all(|d| d.ends_with("です")),
            "distractors must keep the rule tail, got {distractors:?}"
        );
        assert!(distractors.contains(&"静かなです".to_string()));
    }

    #[test]
    fn pure_i_adjective_chain_is_not_valid_form_soup() {
        // Arrange — ～く (adverbial): [AdjectiveToKu]; correct 高く.
        let actions = vec![FormatAction::AdjectiveToKu {}];

        // Act
        let distractors = distractors_for(actions, "高い", PartOfSpeech::IAdjective, "高く");

        // Assert — no valid forms of other rules (form soup), only
        // broken near-misses of 高く itself.
        assert_eq!(distractors.len(), 3);
        for soup in ["高くない", "高かった", "高くて", "高い"] {
            assert!(
                !distractors.iter().any(|d| d == soup),
                "distractors must not be valid other forms, found {soup} in {distractors:?}"
            );
        }
        assert!(
            distractors.iter().all(|d| d.starts_with("高")),
            "distractors must stay within the word, got {distractors:?}"
        );
    }

    #[test]
    fn pure_te_form_chain_breaks_stem_not_ending() {
        // Arrange — te-form: [VerbToTeForm]; correct 書いて.
        let actions = vec![FormatAction::VerbToTeForm {}];

        // Act
        let distractors = distractors_for(actions, "書く", PartOfSpeech::Verb, "書いて");

        // Assert — forgot (書くて), っ confusion (書って), voicing (書いで):
        // the ending て survives in every option.
        assert!(
            distractors.iter().all(|d| d.ends_with("て")),
            "distractors must keep the て ending, got {distractors:?}"
        );
        assert_eq!(distractors.len(), 3);
    }

    #[test]
    fn ichidan_pure_chain_breaks_stem_not_ending() {
        // Arrange — ～ます on an ichidan verb: 食べます.
        let actions = vec![FormatAction::VerbToMasu {}];

        // Act
        let distractors = distractors_for(actions, "食べる", PartOfSpeech::Verb, "食べます");

        // Assert — forgot-to-drop-る (食べるます) is the classic ichidan
        // mistake; every option ends with ます.
        assert!(
            distractors.iter().all(|d| d.ends_with("ます")),
            "distractors must keep the ます ending, got {distractors:?}"
        );
        assert!(distractors.contains(&"食べるます".to_string()));
    }
}

mod generate_grammar_practice_questions {
    use super::*;
    use crate::dictionary::grammar::{FormatAction, GrammarRule, GrammarRuleContent, Nuances};
    use crate::domain::knowledge::VocabularyCard;
    use crate::domain::memory::MemoryState;
    use crate::domain::value_objects::{NativeLanguage, Question};
    use crate::domain::{JapaneseLevel, Rating};
    use std::collections::HashMap;
    use ulid::Ulid;

    fn create_verb_rule() -> GrammarRule {
        GrammarRule::new(
            Ulid::new(),
            JapaneseLevel::N5,
            HashMap::from([(
                NativeLanguage::English,
                GrammarRuleContent::new(
                    "Test Masu".to_string(),
                    "Test desc".to_string(),
                    "Explanation".to_string(),
                    "How to form".to_string(),
                    "Examples".to_string(),
                    Nuances::default(),
                    "Pro tip".to_string(),
                    Vec::new(),
                ),
            )]),
            Some(HashMap::from([(
                PartOfSpeech::Verb,
                vec![FormatAction::VerbToMasu {}],
            )])),
        )
    }

    fn create_known_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn add_known_vocab(ks: &mut KnowledgeSet, word: &str) {
        let mut study_card = ks.create_card(create_known_vocab_card(word)).unwrap();

        let memory = MemoryState::new(
            crate::domain::memory::Stability::new(15.0).unwrap(),
            crate::domain::memory::Difficulty::new(2.0).unwrap(),
            chrono::Utc::now(),
        );
        study_card.apply_review(memory, Rating::Good);
    }

    #[test]
    fn returns_error_when_no_supported_pos() {
        let rule = GrammarRule::new(
            Ulid::new(),
            JapaneseLevel::N5,
            HashMap::from([(
                NativeLanguage::English,
                GrammarRuleContent::new(
                    "Empty".to_string(),
                    "No POS".to_string(),
                    "Explanation".to_string(),
                    "How to form".to_string(),
                    "Examples".to_string(),
                    Nuances::default(),
                    "Pro tip".to_string(),
                    Vec::new(),
                ),
            )]),
            None,
        );

        let ks = KnowledgeSet::new();
        let result = generate_grammar_practice_questions(&rule, &ks, 3, &mut rand::rng());

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OrigaError::GrammarFormatError { .. }
        ));
    }

    #[test]
    fn returns_empty_for_empty_knowledge_set() {
        crate::use_cases::init_real_dictionaries();

        let rule = create_verb_rule();
        let ks = KnowledgeSet::new();
        let result = generate_grammar_practice_questions(&rule, &ks, 3, &mut rand::rng());

        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn practice_questions_have_unique_words() {
        crate::use_cases::init_real_dictionaries();

        let rule = create_verb_rule();
        let mut ks = KnowledgeSet::new();

        for word in ["行く", "食べる", "飲む", "読む", "書く"] {
            add_known_vocab(&mut ks, word);
        }

        let questions =
            generate_grammar_practice_questions(&rule, &ks, 5, &mut rand::rng()).unwrap();

        let words: HashSet<&str> = questions.iter().map(|q| q.word_text()).collect();
        assert_eq!(
            words.len(),
            questions.len(),
            "All word texts must be unique"
        );
    }

    #[test]
    fn returns_fewer_when_not_enough_words() {
        crate::use_cases::init_real_dictionaries();

        let rule = create_verb_rule();
        let mut ks = KnowledgeSet::new();

        add_known_vocab(&mut ks, "行く");
        add_known_vocab(&mut ks, "食べる");

        let questions =
            generate_grammar_practice_questions(&rule, &ks, 10, &mut rand::rng()).unwrap();

        assert!(
            questions.len() <= 2,
            "Expected at most 2 questions, got {}",
            questions.len()
        );
    }

    #[test]
    fn correct_answer_is_at_correct_index() {
        crate::use_cases::init_real_dictionaries();

        let rule = create_verb_rule();
        let mut ks = KnowledgeSet::new();

        for word in ["行く", "食べる", "飲む"] {
            add_known_vocab(&mut ks, word);
        }

        let questions =
            generate_grammar_practice_questions(&rule, &ks, 3, &mut rand::rng()).unwrap();

        for q in &questions {
            let correct = &q.options[q.correct_index];
            let expected = rule.format(q.word_text(), &PartOfSpeech::Verb).unwrap();
            assert_eq!(
                correct, &expected,
                "Option at correct_index should be the formatted word"
            );
        }
    }

    #[test]
    fn each_question_has_four_options() {
        crate::use_cases::init_real_dictionaries();

        let rule = create_verb_rule();
        let mut ks = KnowledgeSet::new();

        for word in ["行く", "食べる", "飲む", "読む"] {
            add_known_vocab(&mut ks, word);
        }

        let questions =
            generate_grammar_practice_questions(&rule, &ks, 4, &mut rand::rng()).unwrap();

        for q in &questions {
            assert_eq!(
                q.options.len(),
                4,
                "Each question must have exactly 4 options (1 correct + 3 distractors)"
            );
        }
    }
}
