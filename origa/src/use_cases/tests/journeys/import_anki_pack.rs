use crate::domain::{NativeLanguage, OrigaError, User};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, init_real_dictionaries};
use crate::use_cases::{
    AnkiCard, ImportAnkiPackUseCase, extract_anki_db_bytes, extract_cards, parse_cards,
    read_anki_database,
};
use serde_json::Value;

// ── extract_anki_db_bytes ─────────────────────────────────────────────

#[test]
fn extract_anki_db_bytes_from_valid_apkg() {
    let apkg_bytes = include_bytes!("../sample.apkg");
    let result = extract_anki_db_bytes(apkg_bytes);

    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

#[test]
fn extract_anki_db_bytes_from_invalid_data() {
    let result = extract_anki_db_bytes(b"not a zip");

    assert!(result.is_err());
    assert!(matches!(result, Err(OrigaError::AnkiInvalidFile { .. })));
}

#[test]
fn extract_anki_db_bytes_from_empty_data() {
    let result = extract_anki_db_bytes(&[]);

    assert!(result.is_err());
    assert!(matches!(result, Err(OrigaError::AnkiInvalidFile { .. })));
}

// ── read_anki_database ────────────────────────────────────────────────

#[test]
fn read_anki_database_from_valid_bytes() {
    let apkg_bytes = include_bytes!("../sample.apkg");
    let db_bytes = extract_anki_db_bytes(apkg_bytes).unwrap();

    let result = read_anki_database(&db_bytes);

    assert!(result.is_ok());
    let deck_info = result.unwrap();
    assert!(!deck_info.detected_fields.is_empty());

    for field in &deck_info.detected_fields {
        eprintln!("sample.apkg field: {} (index {})", field.name, field.index);
    }
}

#[test]
fn read_anki_database_from_invalid_bytes() {
    let result = read_anki_database(b"not a database");

    assert!(result.is_err());
    assert!(matches!(result, Err(OrigaError::AnkiInvalidFile { .. })));
}

// ── parse_cards ───────────────────────────────────────────────────────

#[test]
fn parse_cards_extracts_word_only() {
    let models = r#"{ "123": { "flds": [
        {"name": "Expression"}, {"name": "Meaning"}
    ] } }"#;
    let notes = vec![
        format!("{}\x1f{}", "日本語", "Japanese"),
        format!("{}\x1f{}", "勉強", "study"),
    ];

    let models: Value = serde_json::from_str(models).unwrap();
    let result = parse_cards(&models, &notes, "expression", None);

    assert!(result.is_ok());
    let cards = result.unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].word, "日本語");
    assert!(cards[0].translation.is_none());
    assert_eq!(cards[1].word, "勉強");
}

#[test]
fn parse_cards_extracts_word_and_translation() {
    let models = r#"{ "123": { "flds": [
        {"name": "Expression"}, {"name": "Meaning"}
    ] } }"#;
    let notes = vec![
        format!("{}\x1f{}", "日本語", "Japanese"),
        format!("{}\x1f{}", "勉強", "to study"),
    ];

    let models: Value = serde_json::from_str(models).unwrap();
    let result = parse_cards(&models, &notes, "expression", Some("meaning"));

    assert!(result.is_ok());
    let cards = result.unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].word, "日本語");
    assert_eq!(cards[0].translation.as_deref(), Some("Japanese"));
    assert_eq!(cards[1].word, "勉強");
    assert_eq!(cards[1].translation.as_deref(), Some("to study"));
}

#[test]
fn parse_cards_skips_empty_words() {
    let models = r#"{ "123": { "flds": [
        {"name": "Expression"}, {"name": "Meaning"}
    ] } }"#;
    let notes = vec![
        format!("{}\x1f{}", "日本語", "Japanese"),
        format!("{}\x1f{}", "", "empty word"),
        format!("{}\x1f{}", "勉強", "study"),
    ];

    let models: Value = serde_json::from_str(models).unwrap();
    let result = parse_cards(&models, &notes, "expression", Some("meaning"));

    assert!(result.is_ok());
    let cards = result.unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].word, "日本語");
    assert_eq!(cards[1].word, "勉強");
}

#[test]
fn parse_cards_strips_html_from_fields() {
    let models = r#"{ "123": { "flds": [
        {"name": "Expression"}, {"name": "Meaning"}
    ] } }"#;
    let notes = vec![format!("{}\x1f{}", "<b>日本語</b>", "<i>Japanese</i>")];

    let models: Value = serde_json::from_str(models).unwrap();
    let result = parse_cards(&models, &notes, "expression", Some("meaning"));

    assert!(result.is_ok());
    let cards = result.unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].word, "日本語");
    assert_eq!(cards[0].translation.as_deref(), Some("Japanese"));
}

#[test]
fn parse_cards_is_case_insensitive() {
    let models = r#"{ "123": { "flds": [
        {"name": "EXPRESSION"}, {"name": "MEANING"}
    ] } }"#;
    let notes = vec![format!("{}\x1f{}", "test", "meaning text")];

    let models: Value = serde_json::from_str(models).unwrap();
    let result = parse_cards(&models, &notes, "expression", Some("meaning"));

    assert!(result.is_ok());
    let cards = result.unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].word, "test");
    assert_eq!(cards[0].translation.as_deref(), Some("meaning text"));
}

#[test]
fn parse_cards_returns_error_for_missing_word_field() {
    let models = r#"{ "123": { "flds": [
        {"name": "Front"}, {"name": "Back"}
    ] } }"#;
    let notes = vec![format!("{}\x1f{}", "hello", "world")];

    let models: Value = serde_json::from_str(models).unwrap();
    let result = parse_cards(&models, &notes, "expression", None);

    assert!(result.is_err());
    assert!(matches!(result, Err(OrigaError::AnkiFieldNotFound { .. })));
}

#[test]
fn parse_cards_handles_empty_notes() {
    let models = r#"{ "123": { "flds": [
        {"name": "Expression"}, {"name": "Meaning"}
    ] } }"#;

    let models: Value = serde_json::from_str(models).unwrap();
    let result = parse_cards(&models, &[], "expression", None);

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// ── extract_cards with real .apkg files ───────────────────────────────

#[test]
fn extract_cards_from_sample_apkg() {
    let apkg_bytes = include_bytes!("../sample.apkg");
    let db_bytes = extract_anki_db_bytes(apkg_bytes).unwrap();
    let deck_info = read_anki_database(&db_bytes).unwrap();

    let word_field = deck_info
        .detected_fields
        .iter()
        .find(|f| {
            let n = f.name.to_lowercase();
            n == "expression" || n == "word" || n == "front"
        })
        .or(deck_info.detected_fields.first())
        .expect("No suitable word field in sample.apkg");

    let result = extract_cards(apkg_bytes, &word_field.name, None);

    assert!(result.is_ok());
    let (cards, fields) = result.unwrap();
    assert!(
        !cards.is_empty(),
        "Expected at least one card from sample.apkg"
    );
    assert!(!fields.is_empty());
    assert!(!cards[0].word.is_empty());

    eprintln!(
        "sample.apkg: {} cards, word field: '{}'",
        cards.len(),
        word_field.name
    );
    for card in &cards {
        eprintln!("  '{}' -> {:?}", card.word, card.translation);
    }
}

#[test]
fn extract_cards_with_translation_from_sample_apkg() {
    let apkg_bytes = include_bytes!("../sample.apkg");
    let db_bytes = extract_anki_db_bytes(apkg_bytes).unwrap();
    let deck_info = read_anki_database(&db_bytes).unwrap();

    let word_field = deck_info
        .detected_fields
        .iter()
        .find(|f| {
            let n = f.name.to_lowercase();
            n == "expression" || n == "word" || n == "front"
        })
        .or(deck_info.detected_fields.first())
        .expect("No suitable word field in sample.apkg");

    let translation_field = deck_info.detected_fields.iter().find(|f| {
        let n = f.name.to_lowercase();
        n == "meaning" || n == "definition" || n == "back" || n == "translation"
    });

    let result = extract_cards(
        apkg_bytes,
        &word_field.name,
        translation_field.map(|f| f.name.as_str()),
    );

    assert!(result.is_ok());
    let (cards, _) = result.unwrap();
    assert!(!cards.is_empty());

    if let Some(trans) = translation_field {
        eprintln!(
            "sample.apkg: word='{}', trans='{}', {} cards",
            word_field.name,
            trans.name,
            cards.len()
        );
        for card in &cards {
            eprintln!("  '{}' -> {:?}", card.word, card.translation);
        }
    }
}

#[test]
fn extract_cards_rejects_invalid_zip() {
    let result = extract_cards(b"not a zip", "Expression", None);

    assert!(result.is_err());
    assert!(matches!(result, Err(OrigaError::AnkiInvalidFile { .. })));
}

#[test]
fn extract_cards_rejects_unknown_field() {
    let apkg_bytes = include_bytes!("../sample.apkg");

    let result = extract_cards(apkg_bytes, "NonExistentField", None);

    assert!(result.is_err());
    assert!(matches!(result, Err(OrigaError::AnkiFieldNotFound { .. })));
}

#[test]
fn extract_cards_rejects_empty_data() {
    let result = extract_cards(&[], "Expression", None);

    assert!(result.is_err());
}

// ── Full pipeline ─────────────────────────────────────────────────────

#[test]
fn full_extract_pipeline_with_sample_apkg() {
    let apkg_bytes = include_bytes!("../sample.apkg");

    let db_bytes =
        extract_anki_db_bytes(apkg_bytes).expect("Failed to extract DB from sample.apkg");

    let deck_info = read_anki_database(&db_bytes).expect("Failed to read database");

    for field in &deck_info.detected_fields {
        eprintln!("Detected field: {} (index {})", field.name, field.index);
    }

    let word_field = deck_info
        .detected_fields
        .first()
        .expect("No fields detected");

    let (cards, fields) =
        extract_cards(apkg_bytes, &word_field.name, None).expect("Failed to extract cards");

    assert!(!cards.is_empty(), "Should have extracted at least one card");
    assert!(!cards[0].word.is_empty());
    assert!(!fields.is_empty());
}

// ── execute (async) integration ───────────────────────────────────────

#[tokio::test]
async fn execute_imports_cards_into_user_collection() {
    init_real_dictionaries();

    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let use_case = ImportAnkiPackUseCase::new(&repo);

    let cards = vec![
        AnkiCard {
            word: "日本語".to_string(),
            translation: Some("Japanese".to_string()),
        },
        AnkiCard {
            word: "勉強".to_string(),
            translation: Some("study".to_string()),
        },
    ];

    let result = use_case.execute(cards).await;

    assert!(result.is_ok());
    let import_result = result.unwrap();
    assert!(import_result.total_created_count > 0);

    let user = repo.get_current_user().await.unwrap().unwrap();
    assert!(!user.knowledge_set().study_cards().is_empty());

    eprintln!(
        "Created: {}, Skipped: {:?}",
        import_result.total_created_count, import_result.skipped_words
    );
}

#[tokio::test]
async fn execute_skips_duplicate_cards() {
    init_real_dictionaries();

    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let use_case = ImportAnkiPackUseCase::new(&repo);

    let cards = vec![
        AnkiCard {
            word: "日本語".to_string(),
            translation: Some("Japanese".to_string()),
        },
        AnkiCard {
            word: "日本語".to_string(),
            translation: Some("Japanese".to_string()),
        },
    ];

    let result = use_case.execute(cards).await;

    assert!(result.is_ok());
    let import_result = result.unwrap();
    assert!(
        !import_result.skipped_words.is_empty(),
        "Duplicate words should be skipped"
    );
}

#[tokio::test]
async fn execute_returns_error_when_no_current_user() {
    let repo = InMemoryUserRepository::new();
    let use_case = ImportAnkiPackUseCase::new(&repo);

    let cards = vec![AnkiCard {
        word: "test".to_string(),
        translation: None,
    }];

    let result = use_case.execute(cards).await;

    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(OrigaError::CurrentUserNotExist { .. })
    ));
}

#[tokio::test]
async fn execute_handles_empty_cards_list() {
    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let use_case = ImportAnkiPackUseCase::new(&repo);

    let result = use_case.execute(vec![]).await;

    assert!(result.is_ok());
    let import_result = result.unwrap();
    assert_eq!(import_result.total_created_count, 0);
    assert!(import_result.skipped_words.is_empty());
}
