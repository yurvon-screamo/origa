use origa::{
    application::UserRepository,
    domain::{OrigaError, User},
};
use std::path::PathBuf;
use tokio_fs_ext as fs;
use ulid::Ulid;
use web_sys::console;

#[derive(Clone)]
pub struct FileSystemUserRepository {}

impl FileSystemUserRepository {
    pub fn new() -> Self {
        Self {}
    }

    fn user_file_path(&self, user_id: Ulid) -> PathBuf {
        PathBuf::from(format!("{}.json", user_id))
    }

    async fn list(&self) -> Result<Vec<User>, OrigaError> {
        let mut users = vec![];
        let mut entries = fs::read_dir(".").await.map_err(|e| {
            let reason = format!("Failed to read users directory: {}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            let reason = format!("Failed to read directory entry: {}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                console::warn_1(&format!("Skipping non-json file: {}", path.display()).into());
                continue;
            }

            let content = fs::read_to_string(&path).await.map_err(|e| {
                let reason = format!("Failed to read user file {}: {}", path.display(), e);
                console::error_1(&reason.clone().into());
                OrigaError::RepositoryError { reason }
            })?;

            let user: User = serde_json::from_str(&content).map_err(|e| {
                let reason = format!("Failed to deserialize user {}: {}", path.display(), e);
                console::error_1(&reason.clone().into());
                OrigaError::RepositoryError { reason }
            })?;

            users.push(user);
        }

        Ok(users)
    }
}

impl UserRepository for FileSystemUserRepository {
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
        console::info_1(&format!("Id: {}", user_id).into());

        let file_path = self.user_file_path(user_id);
        console::info_1(&format!("file_path: {}", file_path.display()).into());

        if !file_path.exists() {
            console::warn_1(&format!("User file not found: {}", file_path.display()).into());
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path).await.map_err(|e| {
            let reason = format!("Failed to read user file {}: {}", file_path.display(), e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        let user: User = serde_json::from_str(&content).map_err(|e| {
            let reason = format!("Failed to deserialize user {}: {}", file_path.display(), e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        console::info_1(&format!("User: {:#?}", user).into());
        Ok(Some(user))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, OrigaError> {
        console::info_1(&format!("Email: {}", email).into());

        let users = self.list().await?;
        Ok(users.into_iter().find(|x| x.email() == email))
    }

    async fn find_by_telegram_id(&self, telegram_id: &u64) -> Result<Option<User>, OrigaError> {
        let users = self.list().await?;
        Ok(users
            .into_iter()
            .find(|x| x.telegram_user_id() == Some(telegram_id)))
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        let file_path = self.user_file_path(user.id());
        console::info_1(&format!("Save file_path: {}", file_path.display()).into());

        let json = serde_json::to_string_pretty(&user).map_err(|e| {
            let reason = format!("Failed to serialize user: {}", e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        fs::write(&file_path, json).await.map_err(|e| {
            let reason = format!("Failed to write user file {}: {}", file_path.display(), e);
            console::error_1(&reason.clone().into());
            OrigaError::RepositoryError { reason }
        })?;

        console::info_1(&format!("Saved file_path: {}", file_path.display()).into());
        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        let file_path = self.user_file_path(user_id);

        if file_path.exists() {
            fs::remove_file(&file_path).await.map_err(|e| {
                let reason = format!("Failed to delete user file {}: {}", file_path.display(), e);
                console::error_1(&reason.clone().into());
                OrigaError::RepositoryError { reason }
            })?;
        } else {
            console::warn_1(
                &format!(
                    "Attempted to delete non-existent user file: {}",
                    file_path.display()
                )
                .into(),
            );
        }

        Ok(())
    }
}
