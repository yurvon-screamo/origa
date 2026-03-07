use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use ulid::Ulid;

use origa::{
    domain::{OrigaError, User},
    traits::UserRepository,
};

use crate::repository::file_repository::FileSystemUserRepository;
use crate::repository::jlpt_content_loader::recalculate_user_jlpt_progress;
use crate::repository::trailbase_repository::TrailBaseUserRepository;

static SYNCED: OnceLock<AtomicBool> = OnceLock::new();

fn is_synced() -> bool {
    SYNCED
        .get_or_init(|| AtomicBool::new(false))
        .load(Ordering::Relaxed)
}

fn set_synced(value: bool) {
    SYNCED
        .get_or_init(|| AtomicBool::new(false))
        .store(value, Ordering::Relaxed);
}

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
}

impl UserRepository for HybridUserRepository {
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
        self.sync_if_needed().await;
        self.local.find_by_id(user_id).await
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, OrigaError> {
        self.sync_if_needed().await;
        self.local.find_by_email(email).await
    }

    async fn find_by_telegram_id(&self, telegram_id: &u64) -> Result<Option<User>, OrigaError> {
        self.sync_if_needed().await;
        self.local.find_by_telegram_id(telegram_id).await
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        let mut user_clone = user.clone();
        user_clone.touch();
        recalculate_user_jlpt_progress(&mut user_clone);

        self.local.save(&user_clone).await?;

        let remote = self.remote.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = remote.save(&user_clone).await;
        });

        Ok(())
    }

    async fn save_sync(&self, user: &User) -> Result<(), OrigaError> {
        let mut user_clone = user.clone();
        user_clone.touch();
        recalculate_user_jlpt_progress(&mut user_clone);

        self.local.save(&user_clone).await?;
        self.remote.save(&user_clone).await?;

        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        let result = self.local.delete(user_id).await;

        let remote = self.remote.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = remote.delete(user_id).await;
        });

        result
    }
}

impl HybridUserRepository {
    async fn sync_if_needed(&self) {
        if is_synced() {
            return;
        }

        if let Ok(Some((remote_user, _record_id))) = self.remote.find_current().await {
            if let Ok(Some(local_user)) = self.local.find_by_id(remote_user.id()).await {
                if remote_user.updated_at() != local_user.updated_at() {
                    let mut merged = local_user;
                    merged.merge(&remote_user);

                    match self.local.save(&merged).await {
                        Ok(_) => tracing::info!("Local user updated from remote"),
                        Err(e) => {
                            tracing::error!("Failed to update local user: {:?}", e)
                        }
                    }

                    match self.remote.save(&merged).await {
                        Ok(_) => tracing::info!("Remove user updated"),
                        Err(e) => {
                            tracing::error!("Failed to update remove user: {:?}", e)
                        }
                    }
                }
            } else {
                match self.local.save(&remote_user).await {
                    Ok(_) => tracing::info!("Remote user saved to local storage"),
                    Err(e) => {
                        tracing::error!("Failed to save remote user to local storage: {:?}", e)
                    }
                }
            }
            set_synced(true);
        }
    }
}
