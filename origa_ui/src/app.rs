use crate::repository::get_session;
use crate::repository::{HybridUserRepository, SupabaseClient, clear_session};
use crate::routes::AppRoutes;
use crate::ui_components::LoadingOverlay;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::{GetUserInfoUseCase, UserRepository};
use origa::domain::{OrigaError, User};
use origa::infrastructure::LlmServiceInvoker;
use origa::load_dictionary;

#[derive(Clone)]
pub struct AuthContext {
    pub client: SupabaseClient,
    pub repository: HybridUserRepository,
    pub current_user: RwSignal<Option<User>>,
    pub is_session_loading: RwSignal<bool>,
    pub is_dictionary_loading: RwSignal<bool>,
}

impl AuthContext {
    pub fn new() -> Self {
        Self {
            client: SupabaseClient::new(),
            repository: HybridUserRepository::new(),
            current_user: RwSignal::new(None),
            is_session_loading: RwSignal::new(true),
            is_dictionary_loading: RwSignal::new(true),
        }
    }

    pub async fn init_session(&self) {
        if let Some(session) = get_session() {
            match self.repository.find_by_email(&session.email).await {
                Ok(Some(user)) => {
                    self.current_user.set(Some(user));
                }
                Ok(None) => {}
                Err(OrigaError::SessionExpired) => {
                    clear_session();
                    self.current_user.set(None);
                }
                Err(_) => {}
            }
        }
        self.is_session_loading.set(false);
    }

    pub async fn init_dictionary(&self) {
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = load_dictionary().await {
            log::error!("Failed to load dictionary: {}", e);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Err(e) = load_dictionary() {
            log::error!("Failed to load dictionary: {}", e);
        }
        self.is_dictionary_loading.set(false);
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn update_current_user(repository: HybridUserRepository, current_user: RwSignal<Option<User>>) {
    spawn_local(async move {
        if let Some(user) = current_user.get_untracked() {
            let user_id = user.id();
            let use_case = GetUserInfoUseCase::new(&repository);
            match use_case.execute(user_id).await {
                Ok(_) => {
                    if let Ok(updated_user) = repository.find_by_email(user.email()).await
                        && let Some(u) = updated_user
                    {
                        current_user.set(Some(u))
                    }
                }
                Err(OrigaError::SessionExpired) => {
                    clear_session();
                    current_user.set(None);
                }
                Err(_) => {}
            }
        }
    });
}

#[component]
pub fn App() -> impl IntoView {
    let auth_context = AuthContext::new();

    provide_context(auth_context.repository.clone());
    provide_context(LlmServiceInvoker::None);
    provide_context(auth_context.current_user);
    provide_context(auth_context.clone());

    let ctx = auth_context.clone();
    spawn_local(async move {
        ctx.init_dictionary().await;
    });

    let ctx = auth_context.clone();
    spawn_local(async move {
        ctx.init_session().await;
    });

    view! {
        <Show when=move || auth_context.is_session_loading.get() || auth_context.is_dictionary_loading.get()>
            <LoadingOverlay />
        </Show>
        <AppRoutes />
    }
}
