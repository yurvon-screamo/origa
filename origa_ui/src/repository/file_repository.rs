use origa::{
    application::UserRepository,
    domain::{OrigaError, User},
};
use std::path::PathBuf;
use tokio_fs_ext as fs;
use ulid::Ulid;

const DATABASE_PATH: &str = "~/.origa";

#[derive(Clone)]
pub struct FileSystemUserRepository {
    users_dir: PathBuf,
}

impl FileSystemUserRepository {
    pub fn new() -> Self {
        Self {
            users_dir: PathBuf::from(DATABASE_PATH),
        }
    }

    async fn ensure_dir(&self) -> Result<(), OrigaError> {
        fs::create_dir_all(&self.users_dir)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!(
                    "Failed to create database directory {}: {}",
                    self.users_dir.display(),
                    e
                ),
            })
    }

    fn user_file_path(&self, user_id: Ulid) -> PathBuf {
        self.users_dir.join(format!("{}.json", user_id))
    }

    async fn list(&self) -> Result<Vec<User>, OrigaError> {
        self.ensure_dir().await?;

        let mut users = vec![];
        let mut entries =
            fs::read_dir(&self.users_dir)
                .await
                .map_err(|e| OrigaError::RepositoryError {
                    reason: format!(
                        "Failed to read users directory {}: {}",
                        self.users_dir.display(),
                        e
                    ),
                })?;

        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| OrigaError::RepositoryError {
                    reason: format!("Failed to read directory entry: {}", e),
                })?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let content =
                fs::read_to_string(&path)
                    .await
                    .map_err(|e| OrigaError::RepositoryError {
                        reason: format!("Failed to read user file {}: {}", path.display(), e),
                    })?;

            let user: User =
                serde_json::from_str(&content).map_err(|e| OrigaError::RepositoryError {
                    reason: format!(
                        "Failed to deserialize user {}: {}",
                        &self.users_dir.display(),
                        e
                    ),
                })?;

            users.push(user);
        }

        Ok(users)
    }
}

impl UserRepository for FileSystemUserRepository {
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
        let file_path = self.user_file_path(user_id);
        if !file_path.exists() {
            return Ok(None);
        }

        let content =
            fs::read_to_string(&file_path)
                .await
                .map_err(|e| OrigaError::RepositoryError {
                    reason: format!("Failed to read user file {}: {}", file_path.display(), e),
                })?;

        let user: User =
            serde_json::from_str(&content).map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to deserialize user {}: {}", file_path.display(), e),
            })?;

        Ok(Some(user))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, OrigaError> {
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
        self.ensure_dir().await?;

        let file_path = self.user_file_path(user.id());
        let json =
            serde_json::to_string_pretty(&user).map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to serialize user: {}", e),
            })?;

        fs::write(&file_path, json)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to write user file {}: {}", file_path.display(), e),
            })?;

        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        let file_path = self.user_file_path(user_id);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .await
                .map_err(|e| OrigaError::RepositoryError {
                    reason: format!("Failed to delete user file {}: {}", file_path.display(), e),
                })?;
        }

        Ok(())
    }
}
