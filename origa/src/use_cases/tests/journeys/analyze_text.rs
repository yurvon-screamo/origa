use rstest::rstest;

use crate::domain::{NativeLanguage, OrigaError, User};
use crate::use_cases::AnalyzeTextForCardsUseCase;
use crate::use_cases::tests::fixtures::{
    InMemoryUserRepository, create_test_vocab_card, real_dictionaries_or_skip,
};

#[tokio::test]
async fn analyze_text_finds_vocabulary_candidates() {
    real_dictionaries_or_skip!();

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
    real_dictionaries_or_skip!();

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
    real_dictionaries_or_skip!();

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
    real_dictionaries_or_skip!();

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
    real_dictionaries_or_skip!();

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
    real_dictionaries_or_skip!();

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
async fn analyze_text_collapses_script_and_inflection_variants_into_one_word() {
    real_dictionaries_or_skip!();

    // Arrange
    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    // Act — three writings of the same word: hiragana, kanji, inflected form.
    let analysis = use_case
        .execute("たべる 食べる たべました".to_string())
        .await
        .unwrap();

    // Assert — the analysis path must agree with the manual entry path:
    // one card candidate, keyed by the SudachiDict lemma (uwuwu#1463 US-1).
    assert_eq!(analysis.words.len(), 1);
    assert_eq!(analysis.total_found, 1);
    assert_eq!(analysis.words[0].base_form, "食べる");
}

#[rstest]
#[case::control("たべる 食べる たべました", "食べる")]
#[case::kaku("かく 書く かいた", "書く")]
#[case::kiku("きく 聞く きいた", "聞く")]
#[case::matsu("まつ 待つ まった", "待つ")]
#[case::omou("おもう 思う おもった", "思う")]
#[case::utau("うたう 歌う うたった", "歌う")]
#[case::yomu("よむ 読む よんだ", "読む")]
#[case::aruku("あるく 歩く あるいた", "歩く")]
#[case::kaku_negative("かかない 書く かきました", "書く")]
#[tokio::test]
async fn analyze_text_collapses_godan_script_and_inflection_variants(
    #[case] input: &str,
    #[case] expected: &str,
) {
    real_dictionaries_or_skip!();

    // Arrange
    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    // Act — the same three writings the words drawer analyses in one pass.
    let analysis = use_case.execute(input.to_string()).await.unwrap();

    // Assert — one card candidate, and it must be the kanji lemma (uwuwu#1463 US-1).
    assert_eq!(analysis.words.len(), 1, "input: {input}");
    assert_eq!(analysis.words[0].base_form, expected, "input: {input}");
}

#[tokio::test]
async fn analyze_text_keeps_independent_words_beside_a_writing_anchor() {
    real_dictionaries_or_skip!();

    // Arrange
    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    // Act — a sentence with a kanji anchor must keep every other article.
    let sentence = use_case
        .execute("本を読むのはほんとうに楽しいです。".to_string())
        .await
        .unwrap();
    let two_sentences = use_case
        .execute("あの子はことがすきです。これをください。".to_string())
        .await
        .unwrap();
    // Even in writing-list mode a longer word sharing the anchor's prefix is independent.
    let list = use_case.execute("本 ほんとう".to_string()).await.unwrap();

    // Assert — NFT-3: 本当, こと, これ survive next to their anchors.
    let bases: Vec<&str> = sentence
        .words
        .iter()
        .map(|w| w.base_form.as_str())
        .collect();
    assert!(bases.contains(&"本"), "{bases:?}");
    assert!(bases.contains(&"本当"), "{bases:?}");
    let bases: Vec<&str> = two_sentences
        .words
        .iter()
        .map(|w| w.base_form.as_str())
        .collect();
    assert!(bases.contains(&"こと"), "{bases:?}");
    assert!(bases.contains(&"此れ"), "{bases:?}"); // これ — самостоятельная статья
    let bases: Vec<&str> = list.words.iter().map(|w| w.base_form.as_str()).collect();
    assert!(bases.contains(&"本当"), "{bases:?}");
}

#[tokio::test]
async fn analyze_text_without_user_returns_error() {
    real_dictionaries_or_skip!();

    let repo = InMemoryUserRepository::new();
    let use_case = AnalyzeTextForCardsUseCase::new(&repo);

    let result = use_case.execute("テスト".to_string()).await;

    assert!(matches!(result, Err(OrigaError::CurrentUserNotExist)));
}

#[tokio::test]
async fn analyze_text_provides_word_details() {
    real_dictionaries_or_skip!();

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
    real_dictionaries_or_skip!();

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
    real_dictionaries_or_skip!();

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
    real_dictionaries_or_skip!();

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
