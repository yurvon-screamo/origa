use crate::domain::{iter_grammar_rules, JapaneseLevel, NativeLanguage, User};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{init_real_dictionaries, InMemoryUserRepository};
use crate::use_cases::CreateGrammarCardUseCase;

#[tokio::test]
async fn grammar_rules_loads_from_real_file() {
    init_real_dictionaries();

    let rules: Vec<_> = iter_grammar_rules().collect();
    assert!(!rules.is_empty(), "Grammar rules should not be empty");

    let n5_rules: Vec<_> = rules
        .iter()
        .filter(|r| r.level() == &JapaneseLevel::N5)
        .collect();
    assert!(!n5_rules.is_empty(), "Should have N5 grammar rules");

    let first_n5 = n5_rules.first().expect("Should have at least one N5 rule");
    assert!(!first_n5
        .content(&NativeLanguage::Russian)
        .title()
        .is_empty());
}

#[tokio::test]
async fn create_grammar_card_creates_from_real_rule() {
    init_real_dictionaries();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CreateGrammarCardUseCase::new(&repo);

    let first_rule = iter_grammar_rules()
        .find(|r| r.level() == &JapaneseLevel::N5)
        .expect("Should have at least one N5 grammar rule");
    let rule_id = *first_rule.rule_id();

    let cards = use_case
        .execute(vec![rule_id])
        .await
        .expect("Failed to execute CreateGrammarCardUseCase");

    assert_eq!(cards.len(), 1);

    let saved_user = repo.get_current_user().await.unwrap().unwrap();
    assert!(saved_user
        .knowledge_set()
        .study_cards()
        .contains_key(cards[0].card_id()));
}
