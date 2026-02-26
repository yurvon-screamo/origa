use crate::repository::get_session;
use crate::repository::{SupabaseClient, SupabaseUserRepository, clear_session};
use crate::routes::AppRoutes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::{GetUserInfoUseCase, UserRepository};
use origa::domain::{OrigaError, User};
use origa::infrastructure::LlmServiceInvoker;

#[derive(Clone)]
pub struct AuthContext {
    pub client: SupabaseClient,
    pub repository: SupabaseUserRepository,
    pub current_user: RwSignal<Option<User>>,
}

impl AuthContext {
    pub fn new() -> Self {
        let client = SupabaseClient::new();
        let repository = SupabaseUserRepository::new();
        let current_user = RwSignal::new(None);

        Self {
            client,
            repository,
            current_user,
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
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn update_current_user(
    repository: SupabaseUserRepository,
    current_user: RwSignal<Option<User>>,
) {
    spawn_local(async move {
        if let Some(user) = current_user.get_untracked() {
            let user_id = user.id();
            let use_case = GetUserInfoUseCase::new(&repository);
            match use_case.execute(user_id).await {
                Ok(_) => {
                    if let Ok(updated_user) = repository.find_by_email(user.email()).await {
                        if let Some(u) = updated_user {
                            current_user.set(Some(u))
                        }
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
        ctx.init_session().await;
    });

    view! {
        <AppRoutes />
    }
}
