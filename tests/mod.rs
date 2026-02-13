use origa::application::UserRepository;
use origa::domain::{
    User,
    value_objects::{JapaneseLevel, NativeLanguage},
};
use origa::settings::ApplicationEnvironment;
use tempfile::TempDir;

#[cfg(test)]
pub async fn create_test_repository() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    // Ignore error if settings already initialized (for parallel test execution)
    let _ = ApplicationEnvironment::from_database_path(db_path);
}

#[cfg(test)]
pub async fn create_test_user() -> User {
    let repository = ApplicationEnvironment::get()
        .get_repository()
        .await
        .unwrap();
    let user = User::new(
        "test_user".to_string(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
        None,
    );
    repository.save(&user).await.unwrap();
    user
}
