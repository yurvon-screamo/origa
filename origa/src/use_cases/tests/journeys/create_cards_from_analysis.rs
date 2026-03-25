use crate::domain::{CardType, NativeLanguage, OrigaError, User};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{init_real_dictionaries, InMemoryUserRepository};
use crate::use_cases::{CreateCardsFromAnalysisUseCase, WordToCreate};

fn create_repo() -> InMemoryUserRepository {
    InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ))
}

#[tokio::test]
async fn create_cards_from_analysis_creates_vocabulary_cards() {
    init_real_dictionaries();
    let repo = create_repo();
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);

    let words = vec![WordToCreate {
        base_form: "あい変わらず".to_string(),
    }];

    let result = use_case.execute(words, None).await.unwrap();

    assert_eq!(result.created_cards.len(), 1);
    assert!(result.skipped_words.is_empty());
    assert!(result.failed_words.is_empty());

    let card_type = CardType::from(result.created_cards.first().unwrap().card());
    assert_eq!(card_type, CardType::Vocabulary);
}

#[tokio::test]
async fn create_cards_from_analysis_creates_multiple_cards() {
    init_real_dictionaries();
    let repo = create_repo();
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);

    let words = vec![
        WordToCreate {
            base_form: "あい変わらず".to_string(),
        },
        WordToCreate {
            base_form: "あい昧".to_string(),
        },
    ];

    let result = use_case.execute(words, None).await.unwrap();

    assert!(!result.created_cards.is_empty());
    assert!(result.failed_words.is_empty());
}

#[tokio::test]
async fn create_cards_from_analysis_returns_error_when_user_not_found() {
    let repo = InMemoryUserRepository::new();
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);

    let words = vec![WordToCreate {
        base_form: "あい変わらず".to_string(),
    }];

    let result = use_case.execute(words, None).await;

    assert!(matches!(
        result,
        Err(OrigaError::CurrentUserNotExist { .. })
    ));
}

#[tokio::test]
async fn create_cards_from_analysis_skips_duplicates() {
    init_real_dictionaries();
    let repo = create_repo();
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);

    let words = vec![WordToCreate {
        base_form: "あい変わらず".to_string(),
    }];

    let first = use_case.execute(words.clone(), None).await.unwrap();
    assert_eq!(first.created_cards.len(), 1);
    assert!(first.skipped_words.is_empty());

    let second = use_case.execute(words, None).await.unwrap();
    assert!(second.created_cards.is_empty());
    assert_eq!(second.skipped_words.len(), 1);
    assert_eq!(second.skipped_words.first().unwrap(), "あい変わらず");
}

#[tokio::test]
async fn create_cards_from_analysis_handles_failed_words() {
    init_real_dictionaries();
    let repo = create_repo();
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);

    let words = vec![
        WordToCreate {
            base_form: "あい変わらず".to_string(),
        },
        WordToCreate {
            base_form: "not_japanese".to_string(),
        },
    ];

    let result = use_case.execute(words, None).await.unwrap();

    assert!(!result.created_cards.is_empty());
    assert!(!result.failed_words.is_empty());
    assert!(result.failed_words.iter().any(|(w, _)| w == "not_japanese"));
}

#[tokio::test]
async fn create_cards_from_analysis_marks_set_as_imported() {
    init_real_dictionaries();
    let repo = create_repo();
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);

    let words = vec![WordToCreate {
        base_form: "あい変わらず".to_string(),
    }];
    let set_id = "test-set-123".to_string();

    let result = use_case
        .execute(words, Some(vec![set_id.clone()]))
        .await
        .unwrap();
    assert!(!result.created_cards.is_empty());

    let user = repo.get_current_user().await.unwrap().unwrap();
    assert!(user.is_set_imported(&set_id));
}

#[tokio::test]
async fn create_cards_from_analysis_marks_multiple_sets_as_imported() {
    init_real_dictionaries();
    let repo = create_repo();
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);

    let words = vec![WordToCreate {
        base_form: "あい変わらず".to_string(),
    }];
    let set_ids = vec![
        "set-1".to_string(),
        "set-2".to_string(),
        "set-3".to_string(),
    ];

    let result = use_case
        .execute(words, Some(set_ids.clone()))
        .await
        .unwrap();
    assert!(!result.created_cards.is_empty());

    let user = repo.get_current_user().await.unwrap().unwrap();
    for set_id in &set_ids {
        assert!(
            user.is_set_imported(set_id),
            "Set {} should be imported",
            set_id
        );
    }
}

#[tokio::test]
async fn create_cards_from_analysis_empty_words_creates_nothing() {
    init_real_dictionaries();
    let repo = create_repo();
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);

    let words: Vec<WordToCreate> = vec![];

    let result = use_case.execute(words, None).await.unwrap();

    assert!(result.created_cards.is_empty());
    assert!(result.skipped_words.is_empty());
    assert!(result.failed_words.is_empty());
}

#[tokio::test]
async fn create_cards_from_analysis_persists_cards_in_repository() {
    init_real_dictionaries();
    let repo = create_repo();
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);

    let words = vec![WordToCreate {
        base_form: "あい変わらず".to_string(),
    }];

    use_case.execute(words, None).await.unwrap();

    let user = repo.get_current_user().await.unwrap().unwrap();
    assert_eq!(user.knowledge_set().study_cards().len(), 1);
}

#[tokio::test]
async fn create_cards_from_analysis_mixed_results() {
    init_real_dictionaries();
    let repo = create_repo();
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);

    let words = vec![
        WordToCreate {
            base_form: "あい変わらず".to_string(),
        },
        WordToCreate {
            base_form: "invalid".to_string(),
        },
    ];

    let first = use_case.execute(words.clone(), None).await.unwrap();
    assert!(!first.created_cards.is_empty());
    assert!(!first.failed_words.is_empty());
    assert!(first.skipped_words.is_empty());

    let second = use_case.execute(words, None).await.unwrap();
    assert!(second.created_cards.is_empty());
    assert!(second.skipped_words.contains(&"あい変わらず".to_string()));
}
