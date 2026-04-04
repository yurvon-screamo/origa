use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError, User};
use crate::traits::UserRepository;
use crate::use_cases::UpdateUserProfileUseCase;
use crate::use_cases::tests::fixtures::InMemoryUserRepository;

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
async fn update_user_profile_updates_fields() {
    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let _user = repo.get_current_user().await.unwrap().unwrap();
    let use_case = UpdateUserProfileUseCase::new(&repo);

    use_case
        .execute(NativeLanguage::English, Default::default(), Some(123456789))
        .await
        .unwrap();

    let updated = repo.get_current_user().await.unwrap().unwrap();
    assert_eq!(updated.native_language(), &NativeLanguage::English);
    assert_eq!(updated.telegram_user_id(), Some(&123456789));
}

#[tokio::test]
async fn update_user_profile_returns_error_for_nonexistent_user() {
    let repo = InMemoryUserRepository::new();
    let use_case = UpdateUserProfileUseCase::new(&repo);

    let result = use_case
        .execute(NativeLanguage::English, Default::default(), None)
        .await;

    assert!(matches!(
        result,
        Err(OrigaError::CurrentUserNotExist { .. })
    ));
}
