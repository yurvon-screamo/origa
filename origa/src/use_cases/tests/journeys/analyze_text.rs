use crate::domain::{NativeLanguage, OrigaError, User};
use crate::use_cases::tests::fixtures::{
    create_test_vocab_card, init_real_dictionaries, InMemoryUserRepository,
};
use crate::use_cases::AnalyzeTextForCardsUseCase;

#[tokio::test]
async fn analyze_text_finds_vocabulary_candidates() {
    init_real_dictionaries();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let analysis = use_case
        .execute("私は猫が好きです。".to_string())
        .await
        .unwrap();

    assert!(!analysis.words.is_empty());
    assert!(analysis.total_found > 0);
    assert_eq!(analysis.total_found, analysis.words.len());
}

#[tokio::test]
async fn analyze_text_empty_returns_empty_analysis() {
    init_real_dictionaries();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let analysis = use_case.execute("".to_string()).await.unwrap();

    assert!(analysis.words.is_empty());
    assert_eq!(analysis.total_found, 0);
    assert_eq!(analysis.known_count, 0);
    assert_eq!(analysis.new_count, 0);
}

#[tokio::test]
async fn analyze_text_only_symbols_returns_empty() {
    init_real_dictionaries();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let analysis = use_case.execute("！？。、".to_string()).await.unwrap();

    assert!(analysis.words.is_empty());
    assert_eq!(analysis.total_found, 0);
}

#[tokio::test]
async fn analyze_text_marks_known_words() {
    init_real_dictionaries();

    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let card = create_test_vocab_card("猫");
    user.create_card(card).expect("Failed to create card");

    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let analysis = use_case
        .execute("私は猫が好きです。".to_string())
        .await
        .unwrap();

    let cat_word = analysis.words.iter().find(|w| w.base_form == "猫");
    assert!(cat_word.is_some());
    assert!(cat_word.unwrap().is_known);
    assert!(analysis.known_count >= 1);
}

#[tokio::test]
async fn analyze_text_counts_known_and_new() {
    init_real_dictionaries();

    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let card = create_test_vocab_card("猫");
    user.create_card(card).expect("Failed to create card");

    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let analysis = use_case
        .execute("私は猫が好きです。".to_string())
        .await
        .unwrap();

    assert_eq!(
        analysis.total_found,
        analysis.known_count + analysis.new_count
    );
    assert!(analysis.known_count >= 1);
    assert!(analysis.new_count >= 1);
}

#[tokio::test]
async fn analyze_text_deduplicates_words() {
    init_real_dictionaries();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let analysis = use_case
        .execute("猫が猫を見た。猫は可愛い。".to_string())
        .await
        .unwrap();

    let cat_count = analysis
        .words
        .iter()
        .filter(|w| w.base_form == "猫")
        .count();
    assert_eq!(cat_count, 1, "Duplicate words should be deduplicated");
}

#[tokio::test]
async fn analyze_text_without_user_returns_error() {
    init_real_dictionaries();

    let repo = InMemoryUserRepository::new();
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let result = use_case.execute("テスト".to_string()).await;

    assert!(matches!(
        result,
        Err(OrigaError::CurrentUserNotExist { .. })
    ));
}

#[tokio::test]
async fn analyze_text_provides_word_details() {
    init_real_dictionaries();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let analysis = use_case
        .execute("私は猫が好きです。".to_string())
        .await
        .unwrap();

    for word in &analysis.words {
        assert!(!word.base_form.is_empty());
        assert!(!word.reading.is_empty());
    }
}

#[tokio::test]
async fn analyze_text_mixed_japanese_and_latin() {
    init_real_dictionaries();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let analysis = use_case
        .execute("Hello世界Goodbye".to_string())
        .await
        .unwrap();

    let has_world = analysis.words.iter().any(|w| w.base_form.contains("世界"));
    assert!(has_world, "Should find Japanese words in mixed text");
}

#[tokio::test]
async fn analyze_text_long_text() {
    init_real_dictionaries();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let long_text = "私は昨日、東京に行きました。東京はとても大きい都市です。\
                     電車で新宿まで行って、友達と会いました。\
                     一緒に寿司を食べました。とても美味しかったです。";

    let analysis = use_case.execute(long_text.to_string()).await.unwrap();

    assert!(analysis.total_found > 5, "Long text should have many words");
    assert_eq!(analysis.words.len(), analysis.total_found);
}

#[tokio::test]
async fn analyze_text_whitespace_only() {
    init_real_dictionaries();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let analysis = use_case.execute("   \t\n  ".to_string()).await.unwrap();

    assert!(analysis.words.is_empty());
}
