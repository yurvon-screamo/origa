use ulid::Ulid;

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError, RateMode, Rating, User};
use crate::traits::{SetType, UserRepository, WellKnownSetLoader};
use crate::use_cases::tests::fixtures::{
    FileWellKnownSetLoader, InMemoryUserRepository, create_test_kanji_card, create_test_vocab_card,
    init_real_dictionaries,
};
use crate::use_cases::{
    CreateKanjiCardUseCase, DeleteCardUseCase, ImportWellKnownSetUseCase, KanjiInfoListUseCase,
    KnowledgeSetCardsUseCase, ListWellKnownSetsUseCase, ToggleFavoriteUseCase,
};

async fn create_repo() -> InMemoryUserRepository {
    InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ))
}

async fn get_user_id(repo: &InMemoryUserRepository) -> Ulid {
    repo.find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id()
}

#[tokio::test]
async fn list_well_known_sets_returns_available_sets() {
    let loader = FileWellKnownSetLoader::new();
    let use_case = ListWellKnownSetsUseCase::new(&loader);

    let sets = use_case.execute().await.unwrap();

    assert!(!sets.is_empty());
    let n5_set = sets.iter().find(|s| s.meta.id == "jlpt_n5");
    assert!(n5_set.is_some());
    assert_eq!(n5_set.unwrap().meta.set_type, SetType::Jlpt);
    assert_eq!(n5_set.unwrap().meta.level, JapaneseLevel::N5);
}

#[tokio::test]
async fn load_well_known_set_n5_returns_words() {
    let loader = FileWellKnownSetLoader::new();

    let set = loader.load_set("jlpt_n5".to_string()).await.unwrap();

    assert_eq!(set.level(), &JapaneseLevel::N5);
    assert!(!set.words().is_empty(), "N5 set should have words");
}

#[tokio::test]
async fn well_known_set_preview_shows_known_words() {
    init_real_dictionaries();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let user_id = user.id();

    let loader = FileWellKnownSetLoader::new();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = ImportWellKnownSetUseCase::new(&repo, &loader);

    let preview = use_case
        .preview_set(user_id, "jlpt_n5".to_string())
        .await
        .unwrap();

    assert!(!preview.words.is_empty(), "Preview should have words");
    assert_eq!(
        preview.known_count, 0,
        "New user should have no known words"
    );
}

#[tokio::test]
async fn kanji_info_list_returns_kanji_for_level() {
    init_real_dictionaries();
    let repo = create_repo().await;
    let user_id = get_user_id(&repo).await;
    let use_case = KanjiInfoListUseCase::new(&repo);

    let kanji_list = use_case.execute(user_id, &JapaneseLevel::N5).await.unwrap();

    assert!(!kanji_list.is_empty());
}

#[tokio::test]
async fn kanji_info_list_excludes_learned_kanji() {
    init_real_dictionaries();
    let user = {
        let mut u = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let card = create_test_kanji_card("一");
        u.create_card(card).unwrap();
        let study_card = u.knowledge_set().study_cards().values().next().unwrap();
        u.rate_card(
            *study_card.card_id(),
            Rating::Easy,
            RateMode::StandardLesson,
        )
        .unwrap();
        u
    };
    let repo = InMemoryUserRepository::with_user(user);
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let use_case = KanjiInfoListUseCase::new(&repo);

    let kanji_list = use_case.execute(user_id, &JapaneseLevel::N5).await.unwrap();

    let learned_kanji: Vec<_> = kanji_list.iter().filter(|k| k.kanji == '一').collect();
    assert!(learned_kanji.is_empty(), "Learned kanji should be excluded");
}

#[tokio::test]
async fn create_kanji_card_creates_card_from_dictionary() {
    init_real_dictionaries();
    let repo = create_repo().await;
    let user_id = get_user_id(&repo).await;
    let use_case = CreateKanjiCardUseCase::new(&repo);

    let cards = use_case
        .execute(user_id, vec!["人".to_string()])
        .await
        .unwrap();

    assert_eq!(cards.len(), 1);
    let updated = repo.find_by_id(user_id).await.unwrap().unwrap();
    assert_eq!(updated.knowledge_set().study_cards().len(), 1);
}

#[tokio::test]
async fn knowledge_set_cards_returns_all_cards() {
    let user = {
        let mut u = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        for i in 0..5 {
            let card = create_test_vocab_card(&format!("word_{}", i));
            u.create_card(card).unwrap();
        }
        u
    };
    let repo = InMemoryUserRepository::with_user(user);
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let use_case = KnowledgeSetCardsUseCase::new(&repo);

    let cards = use_case.execute(user_id).await.unwrap();

    assert_eq!(cards.len(), 5);
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
    let use_case = ToggleFavoriteUseCase::new(&repo);

    let is_favorite = use_case.execute(user_id, card_id).await.unwrap();

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
    let use_case = ToggleFavoriteUseCase::new(&repo);

    let first = use_case.execute(user_id, card_id).await.unwrap();
    let second = use_case.execute(user_id, card_id).await.unwrap();

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
    let use_case = DeleteCardUseCase::new(&repo);

    use_case.execute(user_id, card_id).await.unwrap();

    let updated = repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(updated.knowledge_set().study_cards().is_empty());
}

#[tokio::test]
async fn delete_nonexistent_card_returns_error() {
    let repo = create_repo().await;
    let user_id = get_user_id(&repo).await;
    let use_case = DeleteCardUseCase::new(&repo);
    let non_existent_card_id = Ulid::new();

    let result = use_case.execute(user_id, non_existent_card_id).await;

    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}

#[tokio::test]
async fn toggle_favorite_nonexistent_card_returns_error() {
    let repo = create_repo().await;
    let user_id = get_user_id(&repo).await;
    let use_case = ToggleFavoriteUseCase::new(&repo);
    let non_existent_card_id = Ulid::new();

    let result = use_case.execute(user_id, non_existent_card_id).await;

    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}
