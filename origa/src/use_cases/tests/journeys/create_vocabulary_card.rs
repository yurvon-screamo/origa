use crate::domain::{CardType, NativeLanguage, OrigaError, User};
use crate::traits::UserRepository;
use crate::use_cases::CreateVocabularyCardUseCase;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, real_dictionaries_or_skip};

fn create_repo() -> InMemoryUserRepository {
    InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ))
}

#[tokio::test]
async fn create_vocabulary_card_creates_card_for_known_word() {
    real_dictionaries_or_skip!();
    let repo = create_repo();
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    let result = use_case.execute("あい変わらず".to_string()).await.unwrap();

    assert_eq!(result.created_cards.len(), 1);
    assert!(result.skipped_no_translation.is_empty());
    assert!(result.skipped_duplicates.is_empty());

    let user = repo.get_current_user().await.unwrap().unwrap();
    assert_eq!(user.knowledge_set().study_cards().len(), 1);

    let study_card = result.created_cards.first().unwrap();
    let card_type = CardType::from(study_card.card());
    assert_eq!(card_type, CardType::Vocabulary);
}

#[tokio::test]
async fn create_vocabulary_card_returns_error_for_nonexistent_user() {
    let repo = InMemoryUserRepository::new();
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    let result = use_case.execute("あい変わらず".to_string()).await;

    assert!(matches!(result, Err(OrigaError::CurrentUserNotExist)));
}

#[tokio::test]
async fn create_vocabulary_card_duplicate_returns_skipped() {
    real_dictionaries_or_skip!();
    let repo = create_repo();
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    let first = use_case.execute("あい変わらず".to_string()).await.unwrap();
    let first_card_count = first.created_cards.len();
    assert!(first_card_count >= 1);
    assert!(first.skipped_duplicates.is_empty());

    let second = use_case.execute("あい変わらず".to_string()).await.unwrap();
    assert!(second.created_cards.is_empty());
    assert!(!second.skipped_duplicates.is_empty());

    let user = repo.get_current_user().await.unwrap().unwrap();
    assert_eq!(user.knowledge_set().study_cards().len(), first_card_count);
}

#[tokio::test]
async fn create_vocabulary_card_collapses_script_and_inflection_variants() {
    real_dictionaries_or_skip!();
    // Arrange
    let repo = create_repo();
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    // Act
    let hiragana = use_case.execute("たべる".to_string()).await.unwrap();
    let kanji = use_case.execute("食べる".to_string()).await.unwrap();
    let inflected = use_case.execute("たべました".to_string()).await.unwrap();

    // Assert
    assert_eq!(hiragana.created_cards.len(), 1);
    assert_eq!(kanji.created_cards.len(), 0);
    assert_eq!(inflected.created_cards.len(), 0);
    assert_eq!(kanji.skipped_duplicates.len(), 1);
    assert_eq!(inflected.skipped_duplicates.len(), 1);
    let user = repo.get_current_user().await.unwrap().unwrap();
    let cards = user.knowledge_set().study_cards();
    assert_eq!(cards.len(), 1);
    assert_eq!(
        cards.values().next().unwrap().card().content_key(),
        "食べる"
    );
}

#[tokio::test]
async fn create_vocabulary_card_skips_non_japanese_text() {
    real_dictionaries_or_skip!();
    let repo = create_repo();
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    let result = use_case.execute("hello world".to_string()).await.unwrap();

    assert!(result.created_cards.is_empty());
    assert!(result.skipped_no_translation.is_empty());
    assert!(result.skipped_duplicates.is_empty());

    let user = repo.get_current_user().await.unwrap().unwrap();
    assert!(user.knowledge_set().study_cards().is_empty());
}

#[tokio::test]
async fn create_vocabulary_card_creates_multiple_cards_from_sentence() {
    real_dictionaries_or_skip!();
    let repo = create_repo();
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    let result = use_case
        .execute("あい変わらずあい昧".to_string())
        .await
        .unwrap();

    assert!(!result.created_cards.is_empty());

    let user = repo.get_current_user().await.unwrap().unwrap();
    assert!(!user.knowledge_set().study_cards().is_empty());
}

#[tokio::test]
async fn create_vocabulary_card_repository_returns_saved_user() {
    real_dictionaries_or_skip!();
    let repo = create_repo();
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    use_case.execute("あい変わらず".to_string()).await.unwrap();

    let user = repo.get_current_user().await.unwrap().unwrap();
    assert_eq!(user.knowledge_set().study_cards().len(), 1);
}

#[tokio::test]
async fn create_vocabulary_card_empty_text_creates_no_cards() {
    real_dictionaries_or_skip!();
    let repo = create_repo();
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    let result = use_case.execute("".to_string()).await.unwrap();

    assert!(result.created_cards.is_empty());
    assert!(result.skipped_no_translation.is_empty());
    assert!(result.skipped_duplicates.is_empty());
}
