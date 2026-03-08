use ulid::Ulid;

use crate::domain::{NativeLanguage, OrigaError, RateMode, Rating, User};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, create_user_with_vocab_cards};
use crate::use_cases::{RateCardUseCase, SelectCardsToLessonUseCase};

#[tokio::test]
async fn select_cards_to_lesson_returns_cards() {
    let user = create_user_with_vocab_cards(5);
    let repo = InMemoryUserRepository::with_user(user);
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let use_case = SelectCardsToLessonUseCase::new(&repo);

    let cards = use_case.execute(user_id).await.unwrap();

    assert!(!cards.is_empty());
}

#[tokio::test]
async fn select_cards_to_lesson_returns_empty_for_empty_knowledge_set() {
    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let use_case = SelectCardsToLessonUseCase::new(&repo);

    let cards = use_case.execute(user_id).await.unwrap();

    assert!(cards.is_empty());
}

#[tokio::test]
async fn rate_card_again_updates_memory() {
    let user = create_user_with_vocab_cards(1);
    let repo = InMemoryUserRepository::with_user(user);
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let card_id = *repo
        .find_by_id(user_id)
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .keys()
        .next()
        .unwrap();
    let use_case = RateCardUseCase::new(&repo);

    use_case
        .execute(user_id, card_id, RateMode::StandardLesson, Rating::Again)
        .await
        .unwrap();

    let updated = repo.find_by_id(user_id).await.unwrap().unwrap();
    let card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!card.memory().is_new());
}

#[tokio::test]
async fn rate_card_hard_updates_memory() {
    let user = create_user_with_vocab_cards(1);
    let repo = InMemoryUserRepository::with_user(user);
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let card_id = *repo
        .find_by_id(user_id)
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .keys()
        .next()
        .unwrap();
    let use_case = RateCardUseCase::new(&repo);

    use_case
        .execute(user_id, card_id, RateMode::StandardLesson, Rating::Hard)
        .await
        .unwrap();

    let updated = repo.find_by_id(user_id).await.unwrap().unwrap();
    let card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!card.memory().is_new());
}

#[tokio::test]
async fn rate_card_good_updates_memory() {
    let user = create_user_with_vocab_cards(1);
    let repo = InMemoryUserRepository::with_user(user);
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let card_id = *repo
        .find_by_id(user_id)
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .keys()
        .next()
        .unwrap();
    let use_case = RateCardUseCase::new(&repo);

    use_case
        .execute(user_id, card_id, RateMode::StandardLesson, Rating::Good)
        .await
        .unwrap();

    let updated = repo.find_by_id(user_id).await.unwrap().unwrap();
    let card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!card.memory().is_new());
}

#[tokio::test]
async fn rate_card_easy_updates_memory() {
    let user = create_user_with_vocab_cards(1);
    let repo = InMemoryUserRepository::with_user(user);
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let card_id = *repo
        .find_by_id(user_id)
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .keys()
        .next()
        .unwrap();
    let use_case = RateCardUseCase::new(&repo);

    use_case
        .execute(user_id, card_id, RateMode::StandardLesson, Rating::Easy)
        .await
        .unwrap();

    let updated = repo.find_by_id(user_id).await.unwrap().unwrap();
    let card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!card.memory().is_new());
}

#[tokio::test]
async fn full_lesson_cycle_updates_history() {
    let user = create_user_with_vocab_cards(3);
    let repo = InMemoryUserRepository::with_user(user);
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let select_use_case = SelectCardsToLessonUseCase::new(&repo);
    let rate_use_case = RateCardUseCase::new(&repo);

    let cards = select_use_case.execute(user_id).await.unwrap();
    for (card_id, _) in cards {
        rate_use_case
            .execute(user_id, card_id, RateMode::StandardLesson, Rating::Good)
            .await
            .unwrap();
    }

    let updated = repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!updated.knowledge_set().lesson_history().is_empty());
}

#[tokio::test]
async fn rate_card_nonexistent_returns_error() {
    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let use_case = RateCardUseCase::new(&repo);
    let non_existent_card_id = Ulid::new();

    let result = use_case
        .execute(
            user_id,
            non_existent_card_id,
            RateMode::StandardLesson,
            Rating::Good,
        )
        .await;

    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}
