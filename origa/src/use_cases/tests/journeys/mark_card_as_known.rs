use ulid::Ulid;

use crate::domain::{NativeLanguage, OrigaError, RateMode, Rating, RatingContext, User};
use crate::traits::UserRepository;
use crate::use_cases::MarkCardAsKnownUseCase;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, create_test_vocab_card};

#[tokio::test]
async fn no_current_user_returns_current_user_not_exist() {
    // Arrange
    let repo = InMemoryUserRepository::new();
    let use_case = MarkCardAsKnownUseCase::new(&repo);
    let card_id = Ulid::new();

    // Act
    let result = use_case.execute(card_id).await;

    // Assert
    assert!(matches!(result, Err(OrigaError::CurrentUserNotExist)));
}

#[tokio::test]
async fn card_not_found_returns_card_not_found_error() {
    // Arrange
    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = MarkCardAsKnownUseCase::new(&repo);
    let nonexistent_id = Ulid::new();

    // Act
    let result = use_case.execute(nonexistent_id).await;

    // Assert
    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}

#[tokio::test]
async fn in_progress_card_gets_marked_as_known() {
    // Arrange
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let card = create_test_vocab_card("猫");
    let study_card = user.create_card(card).unwrap();
    let card_id = *study_card.card_id();
    user.rate_card(
        card_id,
        Rating::Good,
        RateMode::StandardLesson,
        RatingContext::Explicit,
    )
    .unwrap();

    let repo = InMemoryUserRepository::with_user(user);
    let use_case = MarkCardAsKnownUseCase::new(&repo);

    // Act
    let result = use_case.execute(card_id).await;

    // Assert
    assert!(result.is_ok());
    let updated = repo.get_current_user().await.unwrap().unwrap();
    let updated_card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!updated_card.memory().is_new());
    assert!(
        updated_card.memory().is_known_card(),
        "in-progress card should become known after mark-as-known"
    );
}

#[tokio::test]
async fn already_learned_card_is_idempotent() {
    // Arrange
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let card = create_test_vocab_card("猫");
    let study_card = user.create_card(card).unwrap();
    let card_id = *study_card.card_id();

    let repo = InMemoryUserRepository::with_user(user);
    let use_case = MarkCardAsKnownUseCase::new(&repo);

    use_case.execute(card_id).await.unwrap();
    let stability_before = repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .get_card(card_id)
        .unwrap()
        .memory()
        .stability()
        .map(|s| s.value());

    // Act: mark again on an already-learned card
    let result = use_case.execute(card_id).await;

    // Assert
    assert!(result.is_ok());
    let stability_after = repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .get_card(card_id)
        .unwrap()
        .memory()
        .stability()
        .map(|s| s.value());
    assert_eq!(
        stability_before, stability_after,
        "already-learned card must not be mutated"
    );
}

#[tokio::test]
async fn new_card_gets_rated_easy_and_memory_updated() {
    // Arrange
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let card = create_test_vocab_card("猫");
    let study_card = user.create_card(card).unwrap();
    let card_id = *study_card.card_id();
    assert!(study_card.memory().is_new());

    let repo = InMemoryUserRepository::with_user(user);
    let use_case = MarkCardAsKnownUseCase::new(&repo);

    // Act
    let result = use_case.execute(card_id).await;

    // Assert
    assert!(result.is_ok());
    let updated = repo.get_current_user().await.unwrap().unwrap();
    let updated_card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!updated_card.memory().is_new());
    assert_eq!(updated_card.memory().easy_review_count(), 1);
}

#[tokio::test]
async fn repeat_mark_known_on_known_card_refreshes_stamp_and_keeps_memory_intact() {
    // Arrange
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let study_card = user.create_card(create_test_vocab_card("既知")).unwrap();
    let card_id = *study_card.card_id();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = MarkCardAsKnownUseCase::new(&repo);

    // Act: первое «Знаю» сидирует память; второе (карта уже известна,
    // штамп искусственно состарен до вчерашнего) обязано обновить отметку
    use_case.execute(card_id).await.unwrap();
    let mut aged = repo.get_current_user().await.unwrap().unwrap();
    aged.knowledge_set_mut()
        .study_cards_mut_for_test()
        .get_mut(&card_id)
        .unwrap()
        .memory_history_mut_for_test()
        .set_marked_known_at_for_test(Some(chrono::Utc::now() - chrono::Duration::hours(25)));
    repo.save(&aged).await.unwrap();
    let before = repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .get_card(card_id)
        .unwrap()
        .memory()
        .clone();

    use_case.execute(card_id).await.unwrap();

    // Assert: штамп снова сегодняшний, память нетронута — use case
    // больше не гвардит известные карты
    let after = repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .get_card(card_id)
        .unwrap()
        .memory()
        .clone();
    assert!(
        after.marked_known_today(chrono::Utc::now()),
        "repeat press must refresh the stamp through the use case"
    );
    assert_eq!(after.reps(), before.reps(), "no extra review on repeat");
    assert_eq!(
        after.stability(),
        before.stability(),
        "stability must not be reset"
    );
}

#[tokio::test]
async fn mark_known_silences_companion_same_day_returns_next_day() {
    // Arrange: слово 日本 тянет кандзи 日 и 本 reverse-компаньонами; 日
    // давно известен и не due — но компаньоны не смотрят на due, поэтому
    // летят в урок (исходная жалоба). «Знаю» на известной карте идёт по
    // refresh-ветке (память не трогается, due не пересоздаётся).
    crate::use_cases::init_real_dictionaries();
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let vocab = user.create_card(create_test_vocab_card("日本")).unwrap();
    let kanji_nichi = user
        .create_card(crate::domain::Card::Kanji(
            crate::domain::KanjiCard::new_test("日".to_string()),
        ))
        .unwrap();
    let kanji_hon = user
        .create_card(crate::domain::Card::Kanji(
            crate::domain::KanjiCard::new_test("本".to_string()),
        ))
        .unwrap();
    // 日: известная (stability 30 > порога известности 21), ревью в
    // будущем (не due)
    user.knowledge_set_mut()
        .study_cards_mut_for_test()
        .get_mut(kanji_nichi.card_id())
        .unwrap()
        .seed_first_review(crate::domain::MemoryState::new(
            crate::domain::Stability::new(30.0).unwrap(),
            crate::domain::Difficulty::new(3.0).unwrap(),
            chrono::Utc::now() + chrono::Duration::days(20),
        ));
    let repo = InMemoryUserRepository::with_user(user);

    // Control: до отметки 日 входит компаньоном (канал не смотрит на due)
    let lesson_before = crate::use_cases::SelectCardsToLessonUseCase::new(&repo)
        .execute(
            crate::domain::NewCardPolicy::Inject,
            &crate::domain::JlptContent::new(),
        )
        .await
        .unwrap();
    assert!(
        lesson_before.contains_key(kanji_nichi.card_id()),
        "known not-due kanji must fly in as a companion before the mark"
    );

    // Act I: «Знаю» на известной карте — refresh-ветка
    MarkCardAsKnownUseCase::new(&repo)
        .execute(*kanji_nichi.card_id())
        .await
        .unwrap();

    let lesson_today = crate::use_cases::SelectCardsToLessonUseCase::new(&repo)
        .execute(
            crate::domain::NewCardPolicy::Inject,
            &crate::domain::JlptContent::new(),
        )
        .await
        .unwrap();

    // Assert I: сегодня 日 заглушен, 本 и слово на месте
    assert!(
        !lesson_today.contains_key(kanji_nichi.card_id()),
        "marked-today kanji must be silent in the companion channel"
    );
    assert!(lesson_today.contains_key(kanji_hon.card_id()));
    assert!(lesson_today.contains_key(vocab.card_id()));

    // Act II: назавтра (time-travel отметки на вчера)
    let mut next_day = repo.get_current_user().await.unwrap().unwrap();
    next_day
        .knowledge_set_mut()
        .study_cards_mut_for_test()
        .get_mut(kanji_nichi.card_id())
        .unwrap()
        .memory_history_mut_for_test()
        .set_marked_known_at_for_test(Some(chrono::Utc::now() - chrono::Duration::hours(25)));
    repo.save(&next_day).await.unwrap();

    let lesson_next_day = crate::use_cases::SelectCardsToLessonUseCase::new(&repo)
        .execute(
            crate::domain::NewCardPolicy::Inject,
            &crate::domain::JlptContent::new(),
        )
        .await
        .unwrap();

    // Assert II: канал компаньонов снова открыт
    assert!(
        lesson_next_day.contains_key(kanji_nichi.card_id()),
        "next day the companion channel must reopen"
    );
}
