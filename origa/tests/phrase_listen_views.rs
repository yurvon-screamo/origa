//! Public-boundary pins for the phrase-listen view refresh (uwuwu#1760).
//!
//! `PHRASE_DATA` is process-global, so the cases share one test binary and
//! serialize on a static lock, following `furigana_tokens_regression.rs`.

use std::sync::Mutex;

use origa::dictionary::phrase::{cache_phrase_details, reset_phrase_data_for_test};
use origa::domain::{
    Card, LessonCard, LessonCardView, LessonData, LessonViewGenerator, NativeLanguage, PhraseCard,
    RateMode, Rating, RatingContext, User, refresh_phrase_views,
};
use rand::RngCore;
use ulid::Ulid;

const PHRASE_A: &str = "00000000000011FR8YW9DXRPTJ";
const PHRASE_B: &str = "0000000000003TJXKAQM1NCC1C";

static STORE_LOCK: Mutex<()> = Mutex::new(());

struct PhraseDataGuard;

impl Drop for PhraseDataGuard {
    fn drop(&mut self) {
        reset_phrase_data_for_test();
    }
}

/// A constant RNG keeps `random::<f32>()` at 0.5: above both the review
/// (0.15) and new (0.50) normal-view thresholds, so cases never sample
/// `Normal` by chance.
struct FixedRng;

impl RngCore for FixedRng {
    fn next_u32(&mut self) -> u32 {
        0x8000_0000
    }

    fn next_u64(&mut self) -> u64 {
        0x8000_0000_0000_0000
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for byte in dest {
            *byte = 0x80;
        }
    }
}

fn lock_phrase_data() -> (std::sync::MutexGuard<'static, ()>, PhraseDataGuard) {
    let guard = STORE_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    reset_phrase_data_for_test();
    (guard, PhraseDataGuard)
}

fn install_two_phrases() {
    let json = format!(
        r#"[{{"i":"{PHRASE_A}","x":"分かんないの、分かんないんだよ","ru":"Я не знаю, я действительно не знаю.","en":"I don't know"}},{{"i":"{PHRASE_B}","x":"別のフレーズ","ru":"Совершенно другой ответ","en":"Another answer"}}]"#
    );
    cache_phrase_details(0, &json).expect("install phrase details");
}

fn user_with_phrases(phrase_ids: &[&str], rate_first: bool) -> (User, Vec<Ulid>) {
    let mut user = User::new(
        "phrase-listen@origa.local".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let mut card_ids = Vec::new();
    for id in phrase_ids {
        let phrase_id = Ulid::from_string(id).expect("valid phrase id");
        let study_card = user
            .create_card(Card::Phrase(PhraseCard::new(phrase_id)))
            .expect("create phrase card");
        let card_id = *study_card.card_id();
        if rate_first && card_ids.is_empty() {
            user.rate_card(
                card_id,
                Rating::Good,
                RateMode::PhraseReview,
                RatingContext::Explicit,
            )
            .expect("rate first phrase card");
        }
        card_ids.push(card_id);
    }
    (user, card_ids)
}

fn lesson_with_normal(card_id: Ulid, card: &Card) -> LessonData {
    LessonData {
        cards: vec![(
            card_id,
            LessonCard::new(card_id, LessonCardView::Normal(card.clone()), false),
        )],
        core_count: 1,
    }
}

fn is_phrase_listen(lesson: &LessonData, card_id: Ulid) -> bool {
    matches!(
        lesson.get(&card_id).expect("card in lesson").view(),
        LessonCardView::PhraseListen { .. }
    )
}

#[test]
fn review_phrase_gains_listen_view_once_details_arrive() {
    let (_lock, _guard) = lock_phrase_data();

    // Arrange: the lesson view was chosen before any phrase detail landed.
    let (user, card_ids) = user_with_phrases(&[PHRASE_A, PHRASE_B], true);
    let first_id = card_ids[0];
    let first_card = user
        .knowledge_set()
        .get_card(first_id)
        .expect("review card")
        .card()
        .clone();
    let mut lesson = lesson_with_normal(first_id, &first_card);
    let mut generator = LessonViewGenerator::new(user.knowledge_set(), NativeLanguage::Russian);
    let before = generator.apply_view(
        user.knowledge_set()
            .get_card(first_id)
            .expect("review card"),
        false,
        &mut FixedRng,
    );
    assert!(matches!(before, LessonCardView::Normal(_)));

    // Act
    install_two_phrases();
    let changed = refresh_phrase_views(
        &mut lesson,
        user.knowledge_set(),
        NativeLanguage::Russian,
        &mut FixedRng,
    );

    // Assert
    assert_eq!(changed, 1);
    assert!(is_phrase_listen(&lesson, first_id));
}

#[test]
fn new_phrase_keeps_its_view_when_details_arrive() {
    let (_lock, _guard) = lock_phrase_data();

    // Arrange
    let (user, card_ids) = user_with_phrases(&[PHRASE_A, PHRASE_B], false);
    let first_id = card_ids[0];
    let first_card = user
        .knowledge_set()
        .get_card(first_id)
        .expect("new card")
        .card()
        .clone();
    let mut lesson = lesson_with_normal(first_id, &first_card);

    // Act
    install_two_phrases();
    let changed = refresh_phrase_views(
        &mut lesson,
        user.knowledge_set(),
        NativeLanguage::Russian,
        &mut FixedRng,
    );

    // Assert
    assert_eq!(changed, 0);
    assert!(!is_phrase_listen(&lesson, first_id));
}

#[test]
fn lone_due_phrase_stays_normal_without_a_distractor() {
    let (_lock, _guard) = lock_phrase_data();

    // Arrange
    let (user, card_ids) = user_with_phrases(&[PHRASE_A], true);
    let first_id = card_ids[0];
    let first_card = user
        .knowledge_set()
        .get_card(first_id)
        .expect("review card")
        .card()
        .clone();
    let mut lesson = lesson_with_normal(first_id, &first_card);
    let json = format!(
        r#"[{{"i":"{PHRASE_A}","x":"分かんないの、分かんないんだよ","ru":"Я не знаю, я действительно не знаю.","en":"I don't know"}}]"#
    );
    cache_phrase_details(0, &json).expect("install single phrase detail");

    // Act
    let changed = refresh_phrase_views(
        &mut lesson,
        user.knowledge_set(),
        NativeLanguage::Russian,
        &mut FixedRng,
    );

    // Assert
    assert_eq!(changed, 0);
    assert!(!is_phrase_listen(&lesson, first_id));
}
