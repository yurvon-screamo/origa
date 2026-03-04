#[path = "mod.rs"]
mod tests;

use origa::application::UserRepository;
use origa::settings::ApplicationEnvironment;
use tests::*;

#[tokio::test]
async fn delete_account_should_remove_user_from_repository() {
    create_test_repository().await;
    let repository = ApplicationEnvironment::get().get_repository().await.unwrap();
    let user = create_test_user().await;

    repository.delete(user.id()).await.unwrap();

    assert!(repository.find_by_id(user.id()).await.unwrap().is_none());
    assert!(repository.find_by_email(user.email()).await.unwrap().is_none());
}
