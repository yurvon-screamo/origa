use std::sync::OnceLock;
use ulid::Ulid;

use crate::dictionary::phrase::{cache_phrase_details, init_phrase_index};
use crate::domain::{Card, NativeLanguage, OrigaError, PhraseCard, StudyCard, User};
use crate::traits::UserRepository;
use crate::use_cases::CreatePhraseCardUseCase;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, init_real_dictionaries};

static PHRASE_INIT: OnceLock<()> = OnceLock::new();

fn init_test_phrases() {
    PHRASE_INIT.get_or_init(|| {
        let index_json = r#"{"v":1,"h":"test","total":4,"phrases":[
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["test","hello"],"c":0},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["test","bye"],"c":0},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HN","t":["test","morning"],"c":0},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HM","t":["test","thanks"],"c":0}
        ]}"#;
        init_phrase_index(index_json).expect("Failed to init phrase index");

        let chunk_json = r#"[
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","x":"test hello","ru":"Privet mir","en":"Hello world"},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","x":"test bye","ru":"Do svidaniya mir","en":"Goodbye world"},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HN","x":"test morning","ru":"Dobroe utro","en":"Good morning"},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HM","x":"test thanks"}
        ]"#;
        cache_phrase_details(0, chunk_json).expect("Failed to cache phrase details");
    });
}

fn phrase_id_hello() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID")
}

fn phrase_id_goodbye() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HK").expect("valid ULID")
}

fn phrase_id_morning() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HN").expect("valid ULID")
}

fn phrase_id_no_translation() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HM").expect("valid ULID")
}

fn create_test_user() -> User {
    User::new(
        "phrase-test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    )
}

fn setup() {
    init_real_dictionaries();
    init_test_phrases();
}

#[tokio::test]
async fn create_phrase_card_from_valid_id() {
    setup();

    let user = create_test_user();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CreatePhraseCardUseCase::new(&repo);

    let cards = use_case
        .execute(vec![phrase_id_hello()])
        .await
        .expect("Failed to create phrase card");

    assert_eq!(cards.len(), 1);

    let saved_user = repo
        .get_current_user()
        .await
        .expect("repo error")
        .expect("user should exist");
    assert!(
        saved_user
            .knowledge_set()
            .study_cards()
            .contains_key(cards[0].card_id()),
        "Card should be in KnowledgeSet"
    );
}

#[tokio::test]
async fn create_phrase_card_invalid_id_returns_error() {
    setup();

    let user = create_test_user();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CreatePhraseCardUseCase::new(&repo);

    let missing_id = Ulid::new();
    let result = use_case.execute(vec![missing_id]).await;

    assert!(
        matches!(result, Err(OrigaError::PhraseNotFound { .. })),
        "Expected PhraseNotFound, got: {:?}",
        result
    );
}

#[tokio::test]
async fn create_phrase_card_duplicate_returns_error() {
    setup();

    let user = create_test_user();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CreatePhraseCardUseCase::new(&repo);

    use_case
        .execute(vec![phrase_id_hello()])
        .await
        .expect("First creation should succeed");

    let result = use_case.execute(vec![phrase_id_hello()]).await;

    assert!(
        matches!(result, Err(OrigaError::DuplicateCard { .. })),
        "Expected DuplicateCard, got: {:?}",
        result
    );
}

#[tokio::test]
async fn create_phrase_card_question_returns_text() {
    setup();

    let phrase_card = PhraseCard::new(phrase_id_hello());
    let question = phrase_card.question().expect("Failed to get question");

    assert_eq!(question, "test hello");
}

#[tokio::test]
async fn create_phrase_card_answer_returns_translation() {
    setup();

    let phrase_card = PhraseCard::new(phrase_id_hello());

    let answer_ru = phrase_card
        .answer(&NativeLanguage::Russian)
        .expect("Failed to get Russian answer");
    assert_eq!(answer_ru, "Privet mir");

    let answer_en = phrase_card
        .answer(&NativeLanguage::English)
        .expect("Failed to get English answer");
    assert_eq!(answer_en, "Hello world");
}

#[tokio::test]
async fn create_phrase_card_answer_fallback_to_text() {
    setup();

    let phrase_card = PhraseCard::new(phrase_id_no_translation());

    let answer = phrase_card
        .answer(&NativeLanguage::Russian)
        .expect("Failed to get answer");

    assert_eq!(
        answer, "test thanks",
        "Answer should fall back to Japanese text when translation is missing"
    );
}

#[tokio::test]
async fn create_multiple_phrase_cards() {
    setup();

    let user = create_test_user();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CreatePhraseCardUseCase::new(&repo);

    let ids = vec![phrase_id_hello(), phrase_id_goodbye(), phrase_id_morning()];
    let cards = use_case
        .execute(ids)
        .await
        .expect("Failed to create multiple phrase cards");

    assert_eq!(cards.len(), 3);

    let saved_user = repo
        .get_current_user()
        .await
        .expect("repo error")
        .expect("user should exist");
    assert_eq!(
        saved_user.knowledge_set().study_cards().len(),
        3,
        "All 3 cards should be stored"
    );
}

#[tokio::test]
async fn phrase_card_serialization_roundtrip() {
    setup();

    let phrase_card = PhraseCard::new(phrase_id_hello());
    let study_card = StudyCard::new(Card::Phrase(phrase_card.clone()));

    let json = serde_json::to_string(&study_card).expect("Failed to serialize");
    let deserialized: StudyCard = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(study_card.card_id(), deserialized.card_id());
    assert_eq!(study_card.card(), deserialized.card());
}

#[tokio::test]
async fn phrase_card_content_key_is_phrase_id() {
    setup();

    let phrase_card = PhraseCard::new(phrase_id_hello());
    let card = Card::Phrase(phrase_card);

    assert_eq!(
        card.content_key(),
        "01KPJ5S3N1DRFFD236Z4EZ03HJ",
        "content_key should return phrase_id as string"
    );
}
