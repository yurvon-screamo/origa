use leptos::prelude::*;
use leptos::task::spawn_local;
use tracing::{error, info};

use crate::core::updater;
use crate::loaders::{load_all_data, load_dictionary};
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
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

fn setup_oauth_listener(ctx: AuthContext) {
    use crate::pages::login::auth_handlers::{
        handle_oauth_callback, handle_oauth_callback_desktop,
    };
    use leptos::wasm_bindgen::JsCast;
    use leptos::wasm_bindgen::prelude::*;

    let ctx_clone = ctx.clone();
    let callback = Closure::<dyn Fn(JsValue)>::new(move |event: JsValue| {
        let url = if let Ok(url_str) = js_sys::Reflect::get(&event, &JsValue::from_str("payload"))
            && let Some(s) = url_str.as_string()
        {
            s
        } else if let Some(s) = event.as_string() {
            s
        } else {
            error!("Invalid deep-link event format: {:?}", event);
            return;
        };

        let ctx = ctx_clone.clone();
        spawn_local(async move {
            let result =
                if url::Url::parse(&url).is_ok_and(|u| u.query_pairs().any(|(k, _)| k == "code")) {
                    handle_oauth_callback_desktop(&url, &ctx).await
                } else if let Some(fragment) = url.split('#').nth(1) {
                    handle_oauth_callback(fragment, &ctx).await
                } else {
                    Err("Неподдерживаемый формат callback URL".to_string())
                };

            match result {
                Ok(_) => {
                    ctx.is_authenticated.set(true);
                    if let Some(window) = web_sys::window()
                        && let Err(e) = window.location().set_href("/home")
                    {
                        error!("Failed to redirect to /home: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("OAuth callback error: {}", e);
                }
            }
        });
    });

    if let Some(window) = web_sys::window() {
        let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")).ok();
        if let Some(tauri_obj) = tauri
            && let Some(event_mod) =
                js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("event")).ok()
            && let Some(listen_fn) =
                js_sys::Reflect::get(&event_mod, &JsValue::from_str("listen")).ok()
            && let Ok(listen_fn) = listen_fn.dyn_into::<js_sys::Function>()
        {
            let event_name = JsValue::from_str("deep-link-received");
            let handler = callback.as_ref().clone();
            let _ = listen_fn.call2(&JsValue::UNDEFINED, &event_name, &handler);
            callback.forget();
            return;
        }
    }

    callback.forget();
}

fn check_url_oauth_callback(ctx: &AuthContext) {
    use crate::pages::login::auth_handlers::get_or_create_profile;
    use crate::repository::TrailBaseClient;
    use gloo_storage::{LocalStorage, Storage};

    let path = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();

    if path == "/login" {
        let search = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default();

        if let Some(code) = search.strip_prefix("?code=") {
            let code = code.split('&').next().unwrap_or(code).to_string();

            let verifier: Option<String> = LocalStorage::get("pkce_verifier").ok();
            LocalStorage::delete("pkce_verifier");

            if let Some(verifier) = verifier {
                let is_oauth_loading = ctx.is_oauth_loading;
                is_oauth_loading.set(true);

                let ctx_clone = ctx.clone();
                spawn_local(async move {
                    let client = TrailBaseClient::new();
                    match client
                        .exchange_auth_code_for_session(&code, &verifier)
                        .await
                    {
                        Ok(session) => {
                            if !session.email.is_empty() {
                                let email = session.email.clone();
                                match get_or_create_profile(&ctx_clone, &email).await {
                                    Ok(_) => {
                                        if let Err(e) =
                                            ctx_clone.repository.merge_current_user().await
                                        {
                                            error!("Failed to merge user after OAuth: {:?}", e);
                                        }
                                        if let Some(window) = web_sys::window()
                                            && let Err(e) = window.location().set_href("/home")
                                        {
                                            error!("Failed to redirect to /home: {:?}", e);
                                        }
                                    }
                                    Err(e) => {
                                        is_oauth_loading.set(false);
                                        error!("Failed to create profile: {}", e);
                                    }
                                }
                            } else {
                                is_oauth_loading.set(false);
                            }
                        }
                        Err(e) => {
                            is_oauth_loading.set(false);
                            error!("Failed to exchange auth code: {:?}", e);
                        }
                    }
                });
            }
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let auth_context = AuthContext::new();

    provide_context(auth_context.repository.clone());
    provide_context(auth_context.clone());

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
            <div class="fixed bottom-4 right-4 bg-accent-olive/90 text-white px-3 py-2 rounded-lg text-sm shadow-lg z-50">
                Загрузка словарей...
            </div>
        </Show>
        <AppRoutes />
    }
}
