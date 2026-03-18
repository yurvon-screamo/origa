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

        let local_user = match local_result {
            Some(data) => data,
            None => {
                tracing::warn!("No local user found");
                return Ok(());
            }
        };

        // Если remote не существует - создаём его из local (first-time sync)
        let remote_user = match remote_result {
            Some(data) => data.0,
            None => {
                tracing::info!("First-time sync: creating remote user from local");
                self.remote.save(&local_user).await?;
                return Ok(()); // merge не нужен, объекты идентичны
            }
        };

        // Обычный merge для существующих пользователей
        let mut local_user = local_user;
        local_user.merge(&remote_user);

        self.local.save(&local_user).await?;
        self.remote.save(&local_user).await?;

        Ok(())
    }
}

impl UserRepository for HybridUserRepository {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        self.local.get_current_user().await
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        let mut user_clone = user.clone();

        recalculate_user_jlpt_progress(&mut user_clone);
        tracing::info!("save: Starting save for user {}", user_clone.id());

        self.local.save(&user_clone).await?;
        tracing::info!("save: Local save completed");

        Ok(())
    }

    async fn save_sync(&self, user: &User) -> Result<(), OrigaError> {
        let mut user_clone = user.clone();
        recalculate_user_jlpt_progress(&mut user_clone);

        self.local.save(&user_clone).await?;
        tracing::info!("save_sync: Local save completed");

        match self.merge_current_user().await {
            Ok(_) => {
                tracing::info!("save_sync: Remote sync completed");
            }
            Err(e) => {
                tracing::error!(
                    "save_sync: Remote sync failed: {:?}. Local save is still valid.",
                    e
                );
            }
        }

        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        tracing::info!("delete: Deleting user {}", user_id);

        self.local.delete(user_id).await?;
        tracing::debug!("delete: Local delete completed");

        match self.remote.delete(user_id).await {
            Ok(_) => tracing::info!("delete: Remote delete completed for user {}", user_id),
            Err(e) => {
                tracing::error!("delete: Remote delete failed for user {}: {:?}", user_id, e);
                return Err(e);
            }
        }

        Ok(())
    }
}
