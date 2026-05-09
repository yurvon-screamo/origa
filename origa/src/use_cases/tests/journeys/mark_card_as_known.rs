use ulid::Ulid;

use crate::domain::{NativeLanguage, OrigaError, RateMode, Rating, User};
use crate::traits::UserRepository;
use crate::use_cases::MarkCardAsKnownUseCase;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, create_test_vocab_card};

#[tokio::test]
async fn no_current_user_returns_current_user_not_exist() {
    // Arrange
    let repo = InMemoryUserRepository::new();
    let use_case = MarkCardAsKnownUseCase::new(&repo);
    let card_id = Ulid::new();

    // Act
    let result = use_case.execute(card_id).await;

    // Assert
    assert!(matches!(result, Err(OrigaError::CurrentUserNotExist)));
}

#[tokio::test]
async fn card_not_found_returns_card_not_found_error() {
    // Arrange
    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = MarkCardAsKnownUseCase::new(&repo);
    let nonexistent_id = Ulid::new();

    // Act
    let result = use_case.execute(nonexistent_id).await;

    // Assert
    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}

#[tokio::test]
async fn already_reviewed_card_returns_ok_without_changes() {
    // Arrange
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let card = create_test_vocab_card("猫");
    let study_card = user.create_card(card).unwrap();
    let card_id = *study_card.card_id();
    user.rate_card(card_id, Rating::Good, RateMode::StandardLesson)
        .unwrap();

    let repo = InMemoryUserRepository::with_user(user);
    let use_case = MarkCardAsKnownUseCase::new(&repo);

    // Act
    let result = use_case.execute(card_id).await;

    // Assert
    assert!(result.is_ok());
    let updated = repo.get_current_user().await.unwrap().unwrap();
    let updated_card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!updated_card.memory().is_new());
}

#[tokio::test]
async fn new_card_gets_rated_easy_and_memory_updated() {
    // Arrange
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let card = create_test_vocab_card("猫");
    let study_card = user.create_card(card).unwrap();
    let card_id = *study_card.card_id();
    assert!(study_card.memory().is_new());

    let repo = InMemoryUserRepository::with_user(user);
    let use_case = MarkCardAsKnownUseCase::new(&repo);

    // Act
    let result = use_case.execute(card_id).await;

    // Assert
    assert!(result.is_ok());
    let updated = repo.get_current_user().await.unwrap().unwrap();
    let updated_card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!updated_card.memory().is_new());
    assert_eq!(updated_card.memory().easy_review_count(), 1);
}
