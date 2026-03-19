use crate::domain::{Card, CardType, NativeLanguage, RadicalCard, User};
use crate::traits::UserRepository;
use crate::use_cases::CreateRadicalCardUseCase;

use super::fixtures::{InMemoryUserRepository, init_real_dictionaries};

fn create_test_user() -> User {
    User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    )
}

#[tokio::test]
async fn test_radical_card_new_valid() {
    init_real_dictionaries();
    let result = RadicalCard::new('一');
    assert!(result.is_ok());
    let card = result.unwrap();
    assert_eq!(card.radical_char(), '一');
}

#[tokio::test]
async fn test_radical_card_new_invalid() {
    let result = RadicalCard::new('あ');
    assert!(result.is_err());
}

#[tokio::test]
async fn test_kanji_examples() {
    init_real_dictionaries();
    let card = RadicalCard::new('一').unwrap();
    let examples = card.kanji_examples();
    assert!(!examples.is_empty());
}

#[tokio::test]
async fn test_create_radical_card_usecase() {
    init_real_dictionaries();
    let user = create_test_user();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CreateRadicalCardUseCase::new(&repo);

    let result = use_case.execute(vec!['一', '｜']).await;
    assert!(result.is_ok());
    let cards = result.unwrap();
    assert_eq!(cards.len(), 2);
}

#[tokio::test]
async fn test_create_radical_card_skips_duplicates() {
    init_real_dictionaries();
    let user = create_test_user();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CreateRadicalCardUseCase::new(&repo);

    let result1 = use_case.execute(vec!['一']).await;
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().len(), 1);

    let result2 = use_case.execute(vec!['一']).await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().len(), 0);
}

#[tokio::test]
async fn test_kanji_creates_radicals_journey() {
    init_real_dictionaries();
    let user = create_test_user();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = crate::use_cases::CreateKanjiCardUseCase::new(&repo);

    // 本 содержит радикал 一
    let result = use_case.execute(vec!["本".to_string()]).await;
    assert!(result.is_ok());
    let cards = result.unwrap();

    assert!(!cards.is_empty());

    let user_after = repo.get_current_user().await.unwrap().unwrap();
    let radical_exists = user_after
        .knowledge_set()
        .study_cards()
        .values()
        .any(|c: &crate::domain::StudyCard| {
            matches!(c.card(), Card::Radical(radical) if radical.radical_char() == '一')
        });

    assert!(
        radical_exists,
        "Radical 一 should be auto-created with kanji 本"
    );
}

#[tokio::test]
async fn test_radical_card_type() {
    init_real_dictionaries();
    let card = RadicalCard::new('一').unwrap();
    let card_type = CardType::from(&Card::Radical(card));
    assert_eq!(card_type, CardType::Radical);
}
