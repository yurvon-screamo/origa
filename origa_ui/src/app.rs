use leptos::prelude::*;
use leptos::task::spawn_local;
use tracing::{error, info};

use crate::core::updater;
use crate::loaders::{load_all_data, load_dictionary};
use crate::pages::login::oauth_listeners::{check_url_oauth_callback, setup_oauth_listener};
use crate::repository::{HybridUserRepository, TrailBaseClient};
use crate::routes::AppRoutes;
use crate::ui_components::LoadingOverlay;
use crate::ui_components::UpdateDrawer;

#[derive(Clone)]
pub struct AuthContext {
    pub client: TrailBaseClient,
    pub repository: HybridUserRepository,
    pub is_session_loading: RwSignal<bool>,
    pub is_authenticated: RwSignal<bool>,
    pub is_oauth_loading: RwSignal<bool>,
    pub is_data_loaded: RwSignal<bool>,
}

impl AuthContext {
    pub fn new() -> Self {
        Self {
            client: TrailBaseClient::new(),
            repository: HybridUserRepository::new(),
            is_session_loading: RwSignal::new(true),
            is_oauth_loading: RwSignal::new(false),
            is_data_loaded: RwSignal::new(false),
            is_authenticated: RwSignal::new(false),
        }
    }

    pub async fn init_dictionary(&self) {
        let (dict_result, data_result) = futures::join!(load_dictionary(), load_all_data());

        if let Err(e) = dict_result {
            error!("Failed to load dictionary: {}", e);
        } else {
            info!("Unidic dictionary loaded");
        }
        if let Err(e) = data_result {
            error!("Failed to load data: {:?}", e);
        } else {
            info!("All data loaded");
        }

        self.is_data_loaded.set(true);
    }

    pub fn check_session(&self) {
        use crate::repository::get_session;

        let is_authenticated = self.is_authenticated;
        let is_session_loading = self.is_session_loading;

        spawn_local(async move {
            if let Some(session) = get_session() {
                let now = (js_sys::Date::now() / 1000.0) as u64;

                if session.expires_at > now {
                    is_authenticated.set(true);
                } else {
                    crate::repository::clear_session();
                }
            }
            is_session_loading.set(false);
        });
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

#[component]
pub fn App() -> impl IntoView {
    let auth_context = AuthContext::new();

    provide_context(auth_context.repository.clone());
    provide_context(auth_context.clone());

    auth_context.check_session();
    check_url_oauth_callback(&auth_context);
    setup_oauth_listener(auth_context.clone());

    let update_info = RwSignal::new(None::<updater::UpdateInfo>);
    let download_progress = RwSignal::new(None::<f32>);

    let update_info_clone = update_info;
    spawn_local(async move {
        if let Some(info) = updater::check_for_updates().await {
            update_info_clone.set(Some(info));
        }
    });

    let on_update = Callback::new(move |_| {
        spawn_local(async move {
            download_progress.set(Some(0.0));

            let result = updater::download_and_install(move |progress| {
                download_progress.set(Some(progress as f32));
            })
            .await;

            if let Err(e) = result {
                error!("Update failed: {}", e);
                download_progress.set(None);
            }
        });
    });

    let ctx = auth_context.clone();
    spawn_local(async move {
        ctx.init_dictionary().await;
    });

    view! {
        {move || update_info.get().map(|info| view! {
            <UpdateDrawer
                current_version=info.current_version
                new_version=info.version
                on_update=on_update
                download_progress=Signal::from(download_progress)
            />
        })}
        <Show when=move || auth_context.is_session_loading.get()>
            <LoadingOverlay message="Проверка авторизации..." />
        </Show>
        <Show when=move || auth_context.is_oauth_loading.get()>
            <LoadingOverlay message="Вход..." />
        </Show>
        <Show when=move || !auth_context.is_data_loaded.get()>
            <LoadingOverlay message="Загрузка словарей..." />
        </Show>
        <AppRoutes />
    }
}
