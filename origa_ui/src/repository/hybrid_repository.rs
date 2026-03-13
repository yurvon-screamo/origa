use leptos::prelude::*;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
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

    pub fn start_polling_sync(
        &self,
        sync_ctx: SyncContext,
        current_user: RwSignal<Option<User>>,
        interval_secs: u64,
    ) {
        if SyncContext::is_background_active() {
            tracing::info!("Polling sync already active");
            return;
        }

        let repo = self.clone();
        let sync_ctx_clone = sync_ctx;
        let current_user_clone = current_user;

            wasm_bindgen_futures::spawn_local(async move {
                tracing::info!("Starting polling sync with interval {}s", interval_secs);

                loop {
                    if !SyncContext::is_background_active() {
                        tracing::info!("Polling sync stopped");
                        break;
                    }

                    if sync_ctx_clone.should_sync(interval_secs)
                        && let Some(user) = current_user_clone.get_untracked()
                    {
                        let user_id = user.id();

                        match repo.sync_stats_internal(user_id).await {
                            Ok(Some(merged_user)) => {
                                current_user_clone.set(Some(merged_user));
                                sync_ctx_clone.complete_sync();
                                tracing::debug!("Polling sync completed");
                            }
                            Ok(None) => {
                                tracing::debug!("Polling sync: no changes");
                                sync_ctx_clone.complete_sync();
                            }
                            Err(e) => {
                                tracing::error!("Polling sync error: {:?}", e);
                                sync_ctx_clone.fail_sync(format!("Sync error: {}", e));
                            }
                        }
                    }

                    let interval_ms = (interval_secs * 1000) as u32;
                    gloo_timers::future::TimeoutFuture::new(interval_ms).await;
                }
            });
    }

    pub fn stop_polling(&self, sync_ctx: &SyncContext) {
        sync_ctx.stop_background_sync();
        tracing::info!("Polling sync stop requested");
    }

    pub fn sync_stats(&self, sync_ctx: SyncContext, current_user: RwSignal<Option<User>>) {
        if sync_ctx.is_syncing.get_untracked() {
            tracing::debug!("sync_stats: Sync already in progress");
            return;
        }

        tracing::info!("sync_stats: Starting manual sync");
        sync_ctx.start_sync();

        let repo = self.clone();
        let sync_ctx_clone = sync_ctx;
        let current_user_clone = current_user;

        wasm_bindgen_futures::spawn_local(async move {
            if let Some(user) = current_user_clone.get_untracked() {
                let user_id = user.id();
                tracing::debug!("sync_stats: Syncing user {}", user_id);

                match repo.sync_stats_internal(user_id).await {
                    Ok(Some(merged_user)) => {
                        tracing::info!(
                            "sync_stats: Manual sync completed, updating current_user signal"
                        );
                        current_user_clone.set(Some(merged_user));
                        sync_ctx_clone.complete_sync();
                    }
                    Ok(None) => {
                        sync_ctx_clone.complete_sync();
                        tracing::debug!("sync_stats: Manual sync completed, no changes");
                    }
                    Err(e) => {
                        tracing::error!("sync_stats: Manual sync error: {:?}", e);
                        sync_ctx_clone.fail_sync(format!("Sync failed: {}", e));
                    }
                }
            } else {
                tracing::debug!("sync_stats: No current user to sync");
                sync_ctx_clone.complete_sync();
            }
        });
    }

    async fn do_sync(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
        tracing::info!("do_sync: Starting sync for user {}", user_id);

        let remote_result = self.remote.find_current().await?;

        let (remote_user, _record_id) = match remote_result {
            Some(data) => data,
            None => {
                tracing::debug!("do_sync: No remote user found");
                return Ok(None);
            }
        };

        tracing::info!(
            "do_sync: Remote user found, remote updated_at={:?}, id={}",
            remote_user.updated_at(),
            remote_user.id()
        );

        let local_user = self.local.find_by_id(user_id).await?;

        match local_user {
            Some(local) if remote_user.updated_at() != local.updated_at() => {
                tracing::info!(
                    "do_sync: Merge needed - local updated_at={:?}, remote updated_at={:?}",
                    local.updated_at(),
                    remote_user.updated_at()
                );

                let mut merged = local;
                merged.merge(&remote_user);

                self.local.save(&merged).await?;
                tracing::debug!("do_sync: Merged user saved to local");

                self.remote.save(&merged).await?;
                tracing::debug!("do_sync: Merged user saved to remote");

                set_synced(true);
                tracing::info!("do_sync: User data merged and synced for user {}", user_id);
                Ok(Some(merged))
            }
            Some(local) => {
                tracing::debug!(
                    "do_sync: User data already in sync, updated_at={:?}",
                    local.updated_at()
                );
                set_synced(true);
                Ok(None)
            }
            None => {
                tracing::info!("do_sync: No local user, saving remote user to local storage");
                self.local.save(&remote_user).await?;
                set_synced(true);
                tracing::info!("do_sync: Remote user saved to local storage for user {}", user_id);
                Ok(Some(remote_user))
            }
        }
    }

    pub async fn force_sync(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
        tracing::info!("force_sync: Forcing sync for user {}", user_id);
        let result = self.do_sync(user_id).await;
        match &result {
            Ok(Some(_)) => tracing::info!("force_sync: Sync completed with changes"),
            Ok(None) => tracing::debug!("force_sync: Sync completed, no changes"),
            Err(e) => tracing::error!("force_sync: Sync failed: {:?}", e),
        }
        result
    }

    async fn sync_stats_internal(&self, user_id: Ulid) -> Result<Option<User>, OrigaError> {
        self.do_sync(user_id).await
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

        tracing::info!("save: Starting save for user {}", user_clone.id());

        self.local.save(&user_clone).await?;
        tracing::debug!("save: Local save completed");

        tracing::info!("save: Starting remote save for user {}", user_clone.id());
        match self.remote.save(&user_clone).await {
            Ok(_) => tracing::info!("save: Remote save completed for user {}", user_clone.id()),
            Err(e) => {
                tracing::error!("save: Remote save failed for user {}: {:?}", user_clone.id(), e);
                return Err(e);
            }
        }

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

impl HybridUserRepository {
    async fn sync_if_needed(&self) {
        if is_synced() {
            tracing::debug!("sync_if_needed: Already synced, skipping");
            return;
        }

        tracing::info!("sync_if_needed: Starting initial sync");

        if let Ok(Some((remote_user, _record_id))) = self.remote.find_current().await {
            tracing::info!(
                "sync_if_needed: Remote user found, id={}, updated_at={:?}",
                remote_user.id(),
                remote_user.updated_at()
            );

            if let Ok(Some(local_user)) = self.local.find_by_id(remote_user.id()).await {
                tracing::debug!(
                    "sync_if_needed: Local user found, updated_at={:?}",
                    local_user.updated_at()
                );

                if remote_user.updated_at() != local_user.updated_at() {
                    tracing::info!("sync_if_needed: Merge needed, merging...");

                    let mut merged = local_user;
                    merged.merge(&remote_user);

                    match self.local.save(&merged).await {
                        Ok(_) => tracing::info!("sync_if_needed: Local user updated from remote"),
                        Err(e) => {
                            tracing::error!("sync_if_needed: Failed to update local user: {:?}", e)
                        }
                    }

                    match self.remote.save(&merged).await {
                        Ok(_) => tracing::info!("sync_if_needed: Remote user updated"),
                        Err(e) => {
                            tracing::error!("sync_if_needed: Failed to update remote user: {:?}", e)
                        }
                    }
                } else {
                    tracing::debug!("sync_if_needed: Local and remote already in sync");
                }
            } else {
                tracing::info!("sync_if_needed: No local user, saving remote to local");
                match self.local.save(&remote_user).await {
                    Ok(_) => tracing::info!("sync_if_needed: Remote user saved to local storage"),
                    Err(e) => {
                        tracing::error!("sync_if_needed: Failed to save remote user to local storage: {:?}", e)
                    }
                }
            }
            set_synced(true);
            tracing::info!("sync_if_needed: Initial sync completed");
        } else {
            tracing::debug!("sync_if_needed: No remote user found");
        }
    }
}
