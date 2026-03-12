use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError, User};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::InMemoryUserRepository;
use crate::use_cases::{GetUserInfoUseCase, UpdateUserProfileUseCase};
use ulid::Ulid;

#[tokio::test]
async fn user_new_creates_default_state() {
    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );

    assert!(!user.id().is_nil());
    assert_eq!(user.email(), "test@example.com");
    assert_eq!(user.username(), "test");
    assert_eq!(user.native_language(), &NativeLanguage::Russian);
    assert_eq!(user.current_japanese_level(), JapaneseLevel::N5);
    assert!(user.knowledge_set().study_cards().is_empty());
}

#[tokio::test]
async fn get_user_info_returns_profile() {
    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let use_case = GetUserInfoUseCase::new(&repo);

    let profile = use_case.execute(user_id).await.unwrap();

    assert_eq!(profile.id, user_id);
    assert_eq!(profile.username, "test");
    assert_eq!(profile.native_language, NativeLanguage::Russian);
    assert_eq!(profile.current_japanese_level, JapaneseLevel::N5);
}

#[tokio::test]
async fn update_user_profile_updates_fields() {
    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let user_id = repo
        .find_by_email("test@example.com")
        .await
        .unwrap()
        .unwrap()
        .id();
    let use_case = UpdateUserProfileUseCase::new(&repo);

    use_case
        .execute(user_id, NativeLanguage::English, Some(123456789))
        .await
        .unwrap();

    let updated = repo.find_by_id(user_id).await.unwrap().unwrap();
    assert_eq!(updated.native_language(), &NativeLanguage::English);
    assert_eq!(updated.telegram_user_id(), Some(&123456789));
}

#[tokio::test]
async fn get_user_info_returns_error_for_nonexistent_user() {
    let repo = InMemoryUserRepository::new();
    let use_case = GetUserInfoUseCase::new(&repo);
    let non_existent_id = Ulid::new();

    let result = use_case.execute(non_existent_id).await;

    assert!(matches!(result, Err(OrigaError::UserNotFound { .. })));
}

#[tokio::test]
async fn update_user_profile_returns_error_for_nonexistent_user() {
    let repo = InMemoryUserRepository::new();
    let use_case = UpdateUserProfileUseCase::new(&repo);
    let non_existent_id = Ulid::new();

    let result = use_case
        .execute(non_existent_id, NativeLanguage::English, None)
        .await;

    assert!(matches!(result, Err(OrigaError::UserNotFound { .. })));
}
