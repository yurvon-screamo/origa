use ulid::Ulid;

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError, SetType, User, WellKnownSetMeta};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{
    InMemoryUserRepository, create_test_vocab_card, init_real_dictionaries,
};
use crate::use_cases::{
    CreateKanjiCardUseCase, CreateVocabularyCardUseCase, DeleteCardUseCase, ToggleFavoriteUseCase,
};

async fn create_repo() -> InMemoryUserRepository {
    InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ))
}

#[tokio::test]
async fn well_known_set_minna_nihongo_serialization() {
    let meta = WellKnownSetMeta {
        id: "minna_n5".to_string(),
        set_type: SetType::from("MinnaNoNihongo"),
        level: JapaneseLevel::N5,
        title_ru: "Minna no Nihongo N5".to_string(),
        title_en: "Minna no Nihongo N5".to_string(),
        desc_ru: "Базовый японский учебник уровень N5".to_string(),
        desc_en: "Basic Japanese textbook N5 level".to_string(),
        word_count: 100,
    };

    let set_type = meta.set_type;
    assert_eq!(set_type, "MinnaNoNihongo");
}

#[tokio::test]
async fn toggle_favorite_sets_favorite_true() {
    let user = {
        let mut u = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let card = create_test_vocab_card("word");
        u.create_card(card).unwrap();
        u
    };
    let repo = InMemoryUserRepository::with_user(user);

    let card_id = *repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .keys()
        .next()
        .unwrap();
    let use_case = ToggleFavoriteUseCase::new(&repo);

    let is_favorite = use_case.execute(card_id).await.unwrap();

    assert!(is_favorite);
}

#[tokio::test]
async fn toggle_favorite_toggles_flag() {
    let user = {
        let mut u = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let card = create_test_vocab_card("word");
        u.create_card(card).unwrap();
        u
    };
    let repo = InMemoryUserRepository::with_user(user);
    let card_id = *repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .keys()
        .next()
        .unwrap();
    let use_case = ToggleFavoriteUseCase::new(&repo);

    let first = use_case.execute(card_id).await.unwrap();
    let second = use_case.execute(card_id).await.unwrap();

    assert!(first);
    assert!(!second);
}

#[tokio::test]
async fn delete_card_removes_from_knowledge_set() {
    let user = {
        let mut u = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let card = create_test_vocab_card("word");
        u.create_card(card).unwrap();
        u
    };
    let repo = InMemoryUserRepository::with_user(user);
    let card_id = *repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .keys()
        .next()
        .unwrap();
    let use_case = DeleteCardUseCase::new(&repo);

    use_case.execute(card_id).await.unwrap();

    let updated = repo.get_current_user().await.unwrap().unwrap();
    assert!(updated.knowledge_set().study_cards().is_empty());
}

#[tokio::test]
async fn delete_nonexistent_card_returns_error() {
    let repo = create_repo().await;
    let use_case = DeleteCardUseCase::new(&repo);
    let non_existent_card_id = Ulid::new();

    let result = use_case.execute(non_existent_card_id).await;

    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}

#[tokio::test]
async fn toggle_favorite_nonexistent_card_returns_error() {
    let repo = create_repo().await;
    let use_case = ToggleFavoriteUseCase::new(&repo);
    let non_existent_card_id = Ulid::new();

    let result = use_case.execute(non_existent_card_id).await;

    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}

#[tokio::test]
async fn create_vocabulary_card_empty_text_returns_empty_result() {
    init_real_dictionaries();
    let repo = create_repo().await;
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    let result = use_case.execute("".to_string()).await.unwrap();

    assert!(result.created_cards.is_empty());
    assert!(result.skipped_no_translation.is_empty());
    assert!(result.skipped_duplicates.is_empty());
}

#[tokio::test]
async fn create_vocabulary_card_whitespace_only_returns_empty_result() {
    init_real_dictionaries();
    let repo = create_repo().await;
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    let result = use_case.execute("   ".to_string()).await.unwrap();

    assert!(result.created_cards.is_empty());
}

#[tokio::test]
async fn create_kanji_card_duplicate_returns_error() {
    init_real_dictionaries();
    let repo = create_repo().await;
    let use_case = CreateKanjiCardUseCase::new(&repo);

    use_case.execute(vec!["人".to_string()]).await.unwrap();
    let result = use_case.execute(vec!["人".to_string()]).await;

    assert!(matches!(result, Err(OrigaError::DuplicateCard { .. })));
}

#[tokio::test]
async fn delete_card_already_deleted_returns_error() {
    let user = {
        let mut u = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let card = create_test_vocab_card("word");
        u.create_card(card).unwrap();
        u
    };
    let repo = InMemoryUserRepository::with_user(user);
    let card_id = *repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .keys()
        .next()
        .unwrap();
    let use_case = DeleteCardUseCase::new(&repo);

    use_case.execute(card_id).await.unwrap();
    let result = use_case.execute(card_id).await;

    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}
