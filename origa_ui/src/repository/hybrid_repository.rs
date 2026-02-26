use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use ulid::Ulid;

use origa::{
    application::UserRepository,
    domain::{OrigaError, User},
};

use super::{FileSystemUserRepository, SupabaseUserRepository};

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

pub struct HybridUserRepository {
    local: FileSystemUserRepository,
    remote: SupabaseUserRepository,
}

impl HybridUserRepository {
    pub async fn new() -> Result<Self, OrigaError> {
        let local = FileSystemUserRepository::new().await?;
        Ok(Self {
            local,
            remote: SupabaseUserRepository::new(),
        })
    }
}

impl UserRepository for HybridUserRepository {
    async fn list(&self) -> Result<Vec<User>, OrigaError> {
        self.sync_if_needed().await;
        self.local.list().await
    }

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

        self.local.save(&user_clone).await?;

        let remote = self.remote.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = remote.save(&user_clone).await;
        });

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

        if let Ok(Some(remote_user)) = self.remote.find_current().await {
            if let Ok(Some(local_user)) = self.local.find_by_id(remote_user.id()).await {
                if remote_user.updated_at() > local_user.updated_at() {
                    let mut merged = local_user;
                    merged.merge(&remote_user);
                    let _ = self.local.save(&merged).await;
                }
            } else {
                let _ = self.local.save(&remote_user).await;
            }
            set_synced(true);
        }
    }
}
