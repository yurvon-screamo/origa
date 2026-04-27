use ulid::Ulid;

use origa::{
    domain::{OrigaError, User},
    traits::UserRepository,
};

use crate::loaders::recalculate_user_jlpt_progress;
use crate::repository::file_repository::FileSystemUserRepository;
use crate::repository::trailbase_repository::TrailBaseUserRepository;

#[derive(Clone)]
pub struct HybridUserRepository {
    local: FileSystemUserRepository,
    remote: TrailBaseUserRepository,
}

impl HybridUserRepository {
    pub fn new() -> Self {
        Self {
            local: FileSystemUserRepository::new(),
            remote: TrailBaseUserRepository::new(),
        }
    }

    pub async fn merge_current_user(&self) -> Result<(), OrigaError> {
        let remote_result = self.remote.find_current().await?;
        let local_result = self.local.get_current_user().await?;

        match (remote_result, local_result) {
            (Some(remote_data), None) => {
                tracing::info!("Creating local user from remote");
                self.save_local_and_sync_remote(&remote_data.0).await?;
            },
            (None, Some(local_user)) => {
                tracing::info!("Creating remote user from local");
                self.remote.save(&local_user).await?;
            },
            (Some(remote_data), Some(mut local_user)) => {
                local_user.merge(&remote_data.0);
                self.save_local_and_sync_remote(&local_user).await?;
            },
            (None, None) => {
                tracing::warn!("No user found locally or remotely");
            },
        }

        Ok(())
    }

    async fn save_local_and_sync_remote(&self, user: &User) -> Result<(), OrigaError> {
        self.local.save(user).await?;

        let updated_user =
            self.local
                .get_current_user()
                .await?
                .ok_or_else(|| OrigaError::RepositoryError {
                    reason: "User not found after local save".to_string(),
                })?;
        self.remote.save(&updated_user).await?;
        Ok(())
    }
}

impl UserRepository for HybridUserRepository {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        self.local.get_current_user().await
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        tracing::info!("save: Starting save for user {}", user.id());
        self.local.save(user).await?;
        tracing::info!("save: Local save completed");
        Ok(())
    }

    async fn save_sync(&self, user: &User) -> Result<(), OrigaError> {
        let mut user_clone = user.clone();
        recalculate_user_jlpt_progress(&mut user_clone);

        self.local.save(&user_clone).await?;
        tracing::info!("save_sync: Local save completed");

        match self.remote.save(&user_clone).await {
            Ok(_) => {
                tracing::info!("save_sync: Remote save completed");
            },
            Err(e) => {
                tracing::error!(
                    "save_sync: Remote save failed: {:?}. Local save is still valid.",
                    e
                );
            },
        }

        match self.merge_current_user().await {
            Ok(_) => {
                tracing::info!("save_sync: Remote sync completed");
            },
            Err(e) => {
                tracing::error!(
                    "save_sync: Remote sync failed: {:?}. Local save is still valid.",
                    e
                );
            },
        }

        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        tracing::info!("delete: Deleting user {}", user_id);

        // Always delete local data first
        if let Err(e) = self.local.delete(user_id).await {
            tracing::error!("delete: Local delete failed: {:?}", e);
            return Err(e);
        }
        tracing::info!("delete: Local delete completed for user {}", user_id);

        // Try remote delete, but don't fail if it doesn't work
        match self.remote.delete(user_id).await {
            Ok(_) => tracing::info!("delete: Remote delete completed for user {}", user_id),
            Err(e) => {
                tracing::error!(
                    "delete: Remote delete failed for user {}: {:?}. Local data deleted.",
                    user_id,
                    e
                );
            },
        }

        Ok(())
    }
}
