use chrono::{Duration, Utc};
use ulid::Ulid;

use crate::domain::{
    Card, GhostRung, GhostState, GrammarRuleCard, JlptContent, NativeLanguage, RateMode, Rating,
    RatingContext, User,
};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, create_test_vocab_card};
use crate::use_cases::{MarkCardAsKnownUseCase, RateCardWithSideEffectsUseCase};

fn user_with_vocab(word: &str) -> (User, Ulid) {
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let study_card = user.create_card(create_test_vocab_card(word)).unwrap();
    (user, *study_card.card_id())
}

fn user_with_grammar_card() -> (User, Ulid, Ulid) {
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let rule_id = Ulid::new();
    let study_card = user
        .create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id)))
        .unwrap();
    let card_id = *study_card.card_id();
    user.knowledge_set_mut()
        .rate_card(
            card_id,
            Rating::Good,
            RateMode::GrammarReview,
            RatingContext::Explicit,
        )
        .unwrap();
    (user, card_id, rule_id)
}

/// Time-travel: открывает окно добивания карты (due — час назад, последний
/// шаг — два часа назад). Реальная лестница двигает время только шагами;
/// тесты сдвигают due_at напрямую, как будто прошло 12ч/1д/3д.
async fn open_ghost_window<R: UserRepository>(repo: &R, card_id: &Ulid, rung: GhostRung) {
    let mut user = repo.get_current_user().await.unwrap().unwrap();
    let now = Utc::now();
    let sc = user
        .knowledge_set_mut()
        .study_cards_mut_for_test()
        .get_mut(card_id)
        .unwrap();
    sc.memory_history_mut_for_test()
        .set_ghost_for_test(Some(GhostState::active(
            rung,
            now - Duration::hours(1),
            now - Duration::hours(2),
        )));
    repo.save(&user).await.unwrap();
}

async fn ghost_of<R: UserRepository>(repo: &R, card_id: &Ulid) -> Option<GhostState> {
    repo.get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .get(card_id)
        .unwrap()
        .memory()
        .ghost()
        .cloned()
}

fn active_rung(ghost: &GhostState) -> GhostRung {
    let GhostState::Active { rung, .. } = ghost else {
        panic!("ghost must be active, got {ghost:?}");
    };
    *rung
}

#[tokio::test]
async fn twice_failed_card_walks_ladder_and_returns_to_normal_pipeline() {
    // Arrange
    let (user, card_id) = user_with_vocab("失敗");
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = RateCardWithSideEffectsUseCase::new(&repo);

    // Act: два подряд-провала на явных показах
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Again, None)
        .await
        .unwrap();
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Again, None)
        .await
        .unwrap();

    // Assert: спавн на первой ступени
    assert_eq!(
        active_rung(&ghost_of(&repo, &card_id).await.unwrap()),
        GhostRung::First
    );

    // Act: три успеха по лестнице (окна открываются тайм-тревелом)
    open_ghost_window(&repo, &card_id, GhostRung::First).await;
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Good, None)
        .await
        .unwrap();
    assert_eq!(
        active_rung(&ghost_of(&repo, &card_id).await.unwrap()),
        GhostRung::Second
    );

    open_ghost_window(&repo, &card_id, GhostRung::Second).await;
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Good, None)
        .await
        .unwrap();
    assert_eq!(
        active_rung(&ghost_of(&repo, &card_id).await.unwrap()),
        GhostRung::Third
    );

    open_ghost_window(&repo, &card_id, GhostRung::Third).await;
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Good, None)
        .await
        .unwrap();

    // Assert: закрыто, карта вернулась в обычный пайплайн (исключение снято)
    let user = repo.get_current_user().await.unwrap().unwrap();
    let sc = user.knowledge_set().study_cards().get(&card_id).unwrap();
    assert!(matches!(
        sc.memory().ghost(),
        Some(GhostState::Resolved { .. })
    ));
    assert!(!sc.memory().has_active_ghost(Utc::now()));
}

#[tokio::test]
async fn again_at_third_rung_restarts_ladder() {
    // Arrange: два успеха накоплены, окно открыто
    let (user, card_id) = user_with_vocab("再");
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = RateCardWithSideEffectsUseCase::new(&repo);
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Again, None)
        .await
        .unwrap();
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Again, None)
        .await
        .unwrap();
    open_ghost_window(&repo, &card_id, GhostRung::Third).await;

    // Act: провал на третьей ступени
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Again, None)
        .await
        .unwrap();

    // Assert: лестница рестартовала, добивание живёт
    let ghost = ghost_of(&repo, &card_id).await.unwrap();
    assert_eq!(active_rung(&ghost), GhostRung::First);
    let GhostState::Active { due_at, .. } = ghost else {
        unreachable!();
    };
    assert!(
        due_at > Utc::now(),
        "first rung shows again in 12h, not instantly"
    );
}

#[tokio::test]
async fn abandoned_ghost_written_off_and_card_returns_to_pipeline() {
    // Arrange: добивание молчало 31 день
    let (user, card_id) = user_with_vocab("捨");
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = RateCardWithSideEffectsUseCase::new(&repo);
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Again, None)
        .await
        .unwrap();
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Again, None)
        .await
        .unwrap();

    let mut user = repo.get_current_user().await.unwrap().unwrap();
    let stale = Utc::now() - Duration::days(31);
    let sc = user
        .knowledge_set_mut()
        .study_cards_mut_for_test()
        .get_mut(&card_id)
        .unwrap();
    sc.memory_history_mut_for_test()
        .set_ghost_for_test(Some(GhostState::active(GhostRung::Second, stale, stale)));
    repo.save(&user).await.unwrap();

    // Act: первый явный рейтинг после простоя запускает ленивый TTL
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Good, None)
        .await
        .unwrap();

    // Assert: списано, исключение из пайплайна снято
    let user = repo.get_current_user().await.unwrap().unwrap();
    let sc = user.knowledge_set().study_cards().get(&card_id).unwrap();
    assert!(matches!(
        sc.memory().ghost(),
        Some(GhostState::Expired { .. })
    ));
    assert!(!sc.memory().has_active_ghost(Utc::now()));
}

#[tokio::test]
async fn dual_rating_does_not_touch_grammar_ghost() {
    // Arrange: карта правила с активным добиванием (закрытое окно —
    // любое состояние должно остаться нетронутым неявным рейтингом)
    let (user, grammar_id, rule_id) = user_with_grammar_card();
    let mut user = user;
    let now = Utc::now();
    let vocab = user.create_card(create_test_vocab_card("単語")).unwrap();
    let vocab_id = *vocab.card_id();
    user.knowledge_set_mut()
        .study_cards_mut_for_test()
        .get_mut(&grammar_id)
        .unwrap()
        .memory_history_mut_for_test()
        .set_ghost_for_test(Some(GhostState::active(
            GhostRung::Second,
            now + Duration::days(1),
            now - Duration::hours(2),
        )));
    let repo = InMemoryUserRepository::with_user(user);
    let ghost_before = ghost_of(&repo, &grammar_id).await.unwrap();
    let reps_before = repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .get(&grammar_id)
        .unwrap()
        .memory()
        .reps();

    // Act: Good на слове-мутации правила (dual rating)
    let use_case = RateCardWithSideEffectsUseCase::new(&repo);
    use_case
        .execute(
            vocab_id,
            RateMode::StandardLesson,
            Rating::Good,
            Some(rule_id),
        )
        .await
        .unwrap();

    // Assert: FSRS карты правила двинулся, добивание нетронуто
    let user = repo.get_current_user().await.unwrap().unwrap();
    let grammar = user.knowledge_set().study_cards().get(&grammar_id).unwrap();
    assert_eq!(grammar.memory().reps(), reps_before + 1);
    assert_eq!(grammar.memory().ghost(), Some(&ghost_before));
}

#[tokio::test]
async fn showing_outside_window_updates_fsrs_only() {
    // Arrange: добивание с закрытым окном (due завтра)
    let (user, card_id) = user_with_vocab("窓");
    let mut user = user;
    let now = Utc::now();
    user.knowledge_set_mut()
        .study_cards_mut_for_test()
        .get_mut(&card_id)
        .unwrap()
        .memory_history_mut_for_test()
        .set_ghost_for_test(Some(GhostState::active(
            GhostRung::Second,
            now + Duration::days(1),
            now - Duration::hours(2),
        )));
    let repo = InMemoryUserRepository::with_user(user);
    let ghost_before = ghost_of(&repo, &card_id).await.unwrap();
    let reps_before = repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .get(&card_id)
        .unwrap()
        .memory()
        .reps();

    // Act: явный показ вне окна
    let use_case = RateCardWithSideEffectsUseCase::new(&repo);
    use_case
        .execute(card_id, RateMode::StandardLesson, Rating::Good, None)
        .await
        .unwrap();

    // Assert: FSRS обновился, добивание не изменилось
    let user = repo.get_current_user().await.unwrap().unwrap();
    let sc = user.knowledge_set().study_cards().get(&card_id).unwrap();
    assert_eq!(sc.memory().reps(), reps_before + 1);
    assert_eq!(sc.memory().ghost(), Some(&ghost_before));
}

#[tokio::test]
async fn quiz_on_own_grammar_card_transitions_ghost_exactly_once() {
    // Arrange: квиз на самой грамматической карте — окно открыто
    let (user, grammar_id, rule_id) = user_with_grammar_card();
    let mut user = user;
    let now = Utc::now();
    user.knowledge_set_mut()
        .study_cards_mut_for_test()
        .get_mut(&grammar_id)
        .unwrap()
        .memory_history_mut_for_test()
        .set_ghost_for_test(Some(GhostState::active(
            GhostRung::First,
            now - Duration::hours(1),
            now - Duration::hours(2),
        )));
    let repo = InMemoryUserRepository::with_user(user);

    // Act: рейтится сама карта правила со своим rule_id
    let use_case = RateCardWithSideEffectsUseCase::new(&repo);
    use_case
        .execute(
            grammar_id,
            RateMode::GrammarReview,
            Rating::Good,
            Some(rule_id),
        )
        .await
        .unwrap();

    // Assert: один рейтинг = один шаг лестницы (First → Second),
    // dual rating не задвоил переход
    assert_eq!(
        active_rung(&ghost_of(&repo, &grammar_id).await.unwrap()),
        GhostRung::Second
    );
}

#[tokio::test]
async fn mark_card_as_known_spawns_no_ghost() {
    // Arrange: карта до первого ревью
    let (user, card_id) = user_with_vocab("既知");
    let repo = InMemoryUserRepository::with_user(user);

    // Act: «уже знаю» — прямой apply_review мимо явного показа
    MarkCardAsKnownUseCase::new(&repo)
        .execute(card_id)
        .await
        .unwrap();

    // Assert: карта известна, добивания нет, счётчик пуст
    let user = repo.get_current_user().await.unwrap().unwrap();
    let sc = user.knowledge_set().study_cards().get(&card_id).unwrap();
    assert!(sc.memory().is_known_card());
    assert_eq!(sc.memory().ghost(), None);
    assert_eq!(sc.memory().consecutive_again(), 0);
}

#[tokio::test]
async fn select_cards_to_lesson_includes_open_ghost_hand() {
    // Arrange: карта с открытым окном добивания + обычный материал
    let (mut user, card_id) = user_with_vocab("選");
    let now = Utc::now();
    // Добивание живёт только у прошедших первое ревью карт
    user.knowledge_set_mut()
        .rate_card(
            card_id,
            Rating::Good,
            RateMode::StandardLesson,
            RatingContext::Explicit,
        )
        .unwrap();
    user.knowledge_set_mut()
        .study_cards_mut_for_test()
        .get_mut(&card_id)
        .unwrap()
        .memory_history_mut_for_test()
        .set_ghost_for_test(Some(GhostState::active(
            GhostRung::First,
            now - Duration::hours(1),
            now - Duration::hours(2),
        )));
    for word in ["alpha", "beta", "gamma"] {
        user.create_card(create_test_vocab_card(word)).unwrap();
    }
    let repo = InMemoryUserRepository::with_user(user);

    // Act
    let lesson = crate::use_cases::SelectCardsToLessonUseCase::new(&repo)
        .execute(crate::domain::NewCardPolicy::Exclude, &JlptContent::new())
        .await
        .unwrap();

    // Assert: добивание вошло в урок
    let ids: Vec<ulid::Ulid> = lesson.cards.iter().map(|(_, lc)| lc.card_id()).collect();
    assert!(
        ids.contains(&card_id),
        "ghost hand card must be selected into the lesson"
    );
}
