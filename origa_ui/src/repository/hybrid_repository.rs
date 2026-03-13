use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use ulid::Ulid;

use origa::{
    domain::{OrigaError, User},
    traits::UserRepository,
};

use crate::repository::file_repository::FileSystemUserRepository;
use crate::repository::jlpt_content_loader::recalculate_user_jlpt_progress;
use crate::repository::sync_context::SyncContext;
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

pub fn reset_sync() {
    set_synced(false);
}

const SYNC_INTERVAL_MS: u32 = 30_000;

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

    pub fn start_background_sync(&self, sync_ctx: SyncContext, current_user: RwSignal<Option<User>>) {
        if SyncContext::is_background_active() {
            tracing::info!("Background sync already active");
            return;
        }

        sync_ctx.start_sync();
        
        let repo = self.clone();
        let sync_ctx_clone = sync_ctx;
        let current_user_clone = current_user;
        
        wasm_bindgen_futures::spawn_local(async move {
            tracing::info!("Starting background sync task");
            
            loop {
                if !SyncContext::is_background_active() {
                    tracing::info!("Background sync stopped");
                    break;
                }

                if let Some(user) = current_user_clone.get_untracked() {
                    let user_id = user.id();
                    
                    match repo.sync_user_data(user_id).await {
                        Ok(synced) => {
                            if synced {
                                sync_ctx_clone.complete_sync();
                                tracing::debug!("Background sync completed");
                            }
                        }
                        Err(e) => {
                            tracing::error!("Background sync error: {:?}", e);
                            sync_ctx_clone.fail_sync(format!("Sync error: {}", e));
                        }
                    }
                }

                let sync_ctx_for_timer = sync_ctx_clone;
                let timer_future = gloo_timers::future::TimeoutFuture::new(SYNC_INTERVAL_MS);
                timer_future.await;
                
                if !SyncContext::is_background_active() {
                    break;
                }
                
                sync_ctx_for_timer.start_sync();
            }
        });
    }

    async fn sync_user_data(&self, user_id: Ulid) -> Result<bool, OrigaError> {
        if let Ok(Some((remote_user, _record_id))) = self.remote.find_current().await {
            let local_user = self.local.find_by_id(user_id).await?;
            
            match local_user {
                Some(local) if remote_user.updated_at() != local.updated_at() => {
                    let mut merged = local;
                    merged.merge(&remote_user);

                    self.local.save(&merged).await?;
                    self.remote.save(&merged).await?;
                    
                    tracing::info!("User data synced and merged");
                    return Ok(true);
                }
                Some(_) => {
                    tracing::debug!("User data already in sync");
                    return Ok(false);
                }
                None => {
                    self.local.save(&remote_user).await?;
                    tracing::info!("Remote user saved to local storage");
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    pub fn sync_stats(&self, sync_ctx: SyncContext) {
        if sync_ctx.is_syncing.get_untracked() {
            tracing::debug!("Sync already in progress");
            return;
        }

        sync_ctx.start_sync();
        
        let repo = self.clone();
        let sync_ctx_clone = sync_ctx;
        
        wasm_bindgen_futures::spawn_local(async move {
            match repo.remote.find_current().await {
                Ok(Some((remote_user, _record_id))) => {
                    let user_id = remote_user.id();
                    
                    match repo.local.find_by_id(user_id).await {
                        Ok(Some(local_user)) => {
                            if remote_user.updated_at() != local_user.updated_at() {
                                let mut merged = local_user;
                                merged.merge(&remote_user);

                                match repo.local.save(&merged).await {
                                    Ok(_) => {
                                        tracing::info!("Sync: local user updated");
                                    }
                                    Err(e) => {
                                        tracing::error!("Sync: failed to update local: {:?}", e);
                                    }
                                }

                                let _ = repo.remote.save(&merged).await;
                            }
                        }
                        Ok(None) => {
                            if let Err(e) = repo.local.save(&remote_user).await {
                                tracing::error!("Sync: failed to save remote user: {:?}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Sync: failed to get local user: {:?}", e);
                        }
                    }
                    
                    set_synced(true);
                    sync_ctx_clone.complete_sync();
                }
                Ok(None) => {
                    tracing::debug!("Sync: no remote user found");
                    sync_ctx_clone.complete_sync();
                }
                Err(e) => {
                    tracing::error!("Sync: failed to get remote user: {:?}", e);
                    sync_ctx_clone.fail_sync(format!("Sync failed: {}", e));
                }
            }
        });
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
                        Ok(_) => tracing::info!("Remote user updated"),
                        Err(e) => {
                            tracing::error!("Failed to update remote user: {:?}", e)
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
