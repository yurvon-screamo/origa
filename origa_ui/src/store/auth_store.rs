use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{OrigaError, User};
use origa::traits::UserRepository;

use crate::pages::login::auth_handlers::get_or_create_profile;
use crate::repository::{HybridUserRepository, TrailBaseClient, clear_session, get_session};

/// AuthStore - centralized authentication state management
/// Single source of truth for:
/// - Current user (None = not authenticated)
/// - Loading states
/// - Auth actions (login, logout, delete)
#[derive(Clone)]
pub struct AuthStore {
    client: TrailBaseClient,
    repository: HybridUserRepository,

    /// Current authenticated user (None = not logged in)
    pub user: RwSignal<Option<User>>,

    /// App is checking session on startup
    pub is_checking_session: RwSignal<bool>,

    /// OAuth flow in progress
    pub is_oauth_loading: RwSignal<bool>,

    /// Sync operation in progress
    pub is_syncing: RwSignal<bool>,

    /// Dictionary/data loading complete
    pub is_data_loaded: RwSignal<bool>,

    /// Logout in progress (prevents race conditions)
    is_logging_out: RwSignal<bool>,

    /// Delete account in progress
    is_deleting_account: RwSignal<bool>,
}

impl AuthStore {
    /// Create new AuthStore with default dependencies
    pub fn new() -> Self {
        Self {
            client: TrailBaseClient::new(),
            repository: HybridUserRepository::new(),
            user: RwSignal::new(None),
            is_checking_session: RwSignal::new(true),
            is_oauth_loading: RwSignal::new(false),
            is_syncing: RwSignal::new(false),
            is_data_loaded: RwSignal::new(false),
            is_logging_out: RwSignal::new(false),
            is_deleting_account: RwSignal::new(false),
        }
    }

    // ========================================
    // Computed / Derived State
    // ========================================

    /// Returns a reactive Memo indicating if user is authenticated
    /// Use this in views: <Show when=move || auth_store.is_authenticated().get()>
    pub fn is_authenticated(&self) -> Memo<bool> {
        let user = self.user;
        Memo::new(move |_| user.with(|u| u.is_some()))
    }

    /// Returns a reactive Memo indicating if we're in loading state
    pub fn is_loading(&self) -> Memo<bool> {
        let is_checking_session = self.is_checking_session;
        let is_oauth_loading = self.is_oauth_loading;
        let is_syncing = self.is_syncing;
        Memo::new(move |_| is_checking_session.get() || is_oauth_loading.get() || is_syncing.get())
    }

    /// Get repository for use cases
    pub fn repository(&self) -> &HybridUserRepository {
        &self.repository
    }

    // ========================================
    // Initialization
    // ========================================

    /// Check existing session from LocalStorage on app start
    pub fn check_session(&self) {
        let user_signal = self.user;
        let is_checking = self.is_checking_session;
        let repository = self.repository.clone();

        spawn_local(async move {
            if let Some(session) = get_session() {
                let now = (js_sys::Date::now() / 1000.0) as u64;

                if session.expires_at > now {
                    match repository.get_current_user().await {
                        Ok(Some(user)) => {
                            user_signal.set(Some(user));
                        }
                        Ok(None) => {
                            if repository.merge_current_user().await.is_ok()
                                && let Ok(Some(user)) = repository.get_current_user().await
                            {
                                user_signal.set(Some(user));
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to load user on session check: {:?}", e);
                        }
                    }
                } else {
                    clear_session();
                }
            }
            is_checking.set(false);
        });
    }

    /// Mark data as loaded (dictionary, etc.)
    pub fn set_data_loaded(&self) {
        self.is_data_loaded.set(true);
    }

    // ========================================
    // Auth Actions (Atomic)
    // ========================================

    /// Login with email/password
    pub async fn login(&self, email: &str, password: &str) -> Result<(), OrigaError> {
        self.is_syncing.set(true);

        let result = self.client.login_with_email_password(email, password).await;

        match result {
            Ok(_) => {
                if let Err(e) = self.repository.merge_current_user().await {
                    tracing::error!("Failed to sync user after login: {:?}", e);
                }

                match self.repository.get_current_user().await {
                    Ok(Some(user)) => {
                        self.user.set(Some(user));
                    }
                    Ok(None) => {
                        let _ = self.repository.merge_current_user().await;
                        if let Ok(Some(user)) = self.repository.get_current_user().await {
                            self.user.set(Some(user));
                        }
                    }
                    Err(e) => {
                        self.is_syncing.set(false);
                        return Err(OrigaError::NetworkError {
                            url: "/api/auth/v1/login".to_string(),
                            reason: format!("Failed to load user: {}", e),
                        });
                    }
                }

                self.is_syncing.set(false);
                Ok(())
            }
            Err(e) => {
                self.is_syncing.set(false);
                Err(OrigaError::NetworkError {
                    url: "/api/auth/v1/login".to_string(),
                    reason: e.to_string(),
                })
            }
        }
    }

    /// Set session after OAuth callback
    pub async fn set_oauth_session(
        &self,
        code: &str,
        pkce_verifier: &str,
    ) -> Result<(), OrigaError> {
        if self.user.with(|u| u.is_some()) {
            self.is_oauth_loading.set(false);
            return Ok(());
        }

        self.is_oauth_loading.set(true);

        match self
            .client
            .exchange_auth_code_for_session(code, pkce_verifier)
            .await
        {
            Ok(session) => {
                if session.email.is_empty() {
                    self.is_oauth_loading.set(false);
                    return Err(OrigaError::NetworkError {
                        url: "/api/auth/v1/token".to_string(),
                        reason: "Email not found in OAuth token".to_string(),
                    });
                }

                match get_or_create_profile(self, &session.email).await {
                    Ok(user) => {
                        self.user.set(Some(user));
                        self.is_oauth_loading.set(false);
                        Ok(())
                    }
                    Err(e) => {
                        self.is_oauth_loading.set(false);
                        Err(OrigaError::InvalidValues { reason: e })
                    }
                }
            }
            Err(e) => {
                self.is_oauth_loading.set(false);
                Err(OrigaError::NetworkError {
                    url: "/api/auth/v1/token".to_string(),
                    reason: e.to_string(),
                })
            }
        }
    }

    /// Logout - atomically clears all auth state
    pub async fn logout(&self) -> Result<(), OrigaError> {
        if self.is_logging_out.get() {
            return Ok(());
        }

        self.is_logging_out.set(true);

        let _ = self.client.logout().await;

        self.clear_auth_state().await;

        self.is_logging_out.set(false);
        Ok(())
    }

    /// Delete account - atomically removes account and all local data
    pub async fn delete_account(&self) -> Result<(), OrigaError> {
        if self.is_deleting_account.get() {
            return Ok(());
        }

        self.is_deleting_account.set(true);

        if let Err(e) = self.client.delete_account().await {
            tracing::error!("Server account delete failed: {:?}", e);
            self.clear_auth_state().await;
            self.is_deleting_account.set(false);
            return Err(OrigaError::AccountDeletionFailed {
                reason: e.to_string(),
            });
        }

        let user_id = self.user.with(|u| u.clone().map(|user| user.id()));
        self.clear_auth_state().await;

        if let Some(id) = user_id
            && let Err(e) = self.repository.delete(id).await
        {
            tracing::error!(
                "Local user data delete failed after successful server deletion: {:?}",
                e
            );
        }

        self.is_deleting_account.set(false);
        Ok(())
    }

    /// Internal: Clear all authentication-related state
    async fn clear_auth_state(&self) {
        clear_session();

        if let Some(user) = self.user.get() {
            let _ = self.repository.delete(user.id()).await;
        }

        self.user.set(None);
        self.is_data_loaded.set(false);
    }

    // ========================================
    // Data Sync
    // ========================================

    /// Refresh user data from local storage
    pub async fn refresh_user(&self) -> Result<(), OrigaError> {
        match self.repository.get_current_user().await {
            Ok(Some(user)) => {
                self.user.set(Some(user));
                Ok(())
            }
            Ok(None) => Err(OrigaError::CurrentUserNotExist {}),
            Err(e) => Err(e),
        }
    }
}

impl Default for AuthStore {
    fn default() -> Self {
        Self::new()
    }
}
