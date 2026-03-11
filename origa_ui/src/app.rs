use crate::data_loader::load_all_data;
use crate::dictionary::load_dictionary;
use crate::repository::get_session;
use crate::repository::{HybridUserRepository, TrailBaseClient, clear_session};
use crate::routes::AppRoutes;
use crate::ui_components::LoadingOverlay;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{OrigaError, User};
use origa::traits::UserRepository;
use origa::use_cases::GetUserInfoUseCase;
use tracing::{error, info};

#[derive(Clone)]
pub struct AuthContext {
    pub client: TrailBaseClient,
    pub repository: HybridUserRepository,
    pub current_user: RwSignal<Option<User>>,
    pub is_session_loading: RwSignal<bool>,
    pub is_dictionary_loading: RwSignal<bool>,
}

impl AuthContext {
    pub fn new() -> Self {
        Self {
            client: TrailBaseClient::new(),
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

fn setup_oauth_listener(ctx: AuthContext) {
    use crate::pages::login::auth_handlers::{
        handle_oauth_callback, handle_oauth_callback_desktop,
    };
    use leptos::wasm_bindgen::JsCast;
    use leptos::wasm_bindgen::prelude::*;

    info!("[ORIGA-UI] Setting up OAuth listener...");

    let ctx_clone = ctx.clone();
    let callback = Closure::<dyn Fn(JsValue)>::new(move |event: JsValue| {
        info!("[ORIGA-UI] Callback triggered with event: {:?}", event);
        
        let url = if let Ok(url_str) = js_sys::Reflect::get(&event, &JsValue::from_str("payload"))
            && let Some(s) = url_str.as_string()
        {
            info!("[ORIGA-UI] Extracted URL from event.payload: {}", s);
            s
        } else if let Some(s) = event.as_string() {
            info!("[ORIGA-UI] Event is direct string: {}", s);
            s
        } else {
            error!("[ORIGA-UI] Invalid deep-link event format: {:?}", event);
            return;
        };

        info!("[ORIGA-UI] Deep link received: {}", url);
        let ctx = ctx_clone.clone();
        spawn_local(async move {
            info!("[ORIGA-UI] Processing deep link URL: {}", url);
            let result =
                if url::Url::parse(&url).is_ok_and(|u| u.query_pairs().any(|(k, _)| k == "code")) {
                    info!("[ORIGA-UI] Using desktop callback handler");
                    handle_oauth_callback_desktop(&url, &ctx).await
                } else if let Some(fragment) = url.split('#').nth(1) {
                    info!("[ORIGA-UI] Using web callback handler");
                    handle_oauth_callback(fragment, &ctx).await
                } else {
                    Err("Неподдерживаемый формат callback URL".to_string())
                };

            match result {
                Ok(user) => {
                    info!("[ORIGA-UI] OAuth success, user: {:?}", user.email());
                    ctx.current_user.set(Some(user));
                    if let Some(window) = web_sys::window()
                        && let Err(e) = window.location().set_href("/home")
                    {
                        error!("[ORIGA-UI] Failed to redirect to /home: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("[ORIGA-UI] OAuth callback error: {}", e);
                }
            }
        });
    });

    if let Some(window) = web_sys::window() {
        info!("[ORIGA-UI] Checking for __TAURI__ object...");
        let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")).ok();
        info!("[ORIGA-UI] __TAURI__ present: {}", tauri.is_some());
        
        if let Some(tauri_obj) = tauri {
            info!("[ORIGA-UI] Checking for event module...");
            let event_mod = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("event")).ok();
            info!("[ORIGA-UI] event module present: {}", event_mod.is_some());
            
            if let Some(event_mod) = event_mod {
                let listen_fn = js_sys::Reflect::get(&event_mod, &JsValue::from_str("listen")).ok();
                info!("[ORIGA-UI] listen function present: {}", listen_fn.is_some());
                
                if let Some(listen_fn) = listen_fn
                    && let Ok(listen_fn) = listen_fn.dyn_into::<js_sys::Function>()
                {
                    let event_name = JsValue::from_str("deep-link-received");
                    let handler = callback.as_ref().clone();
                    let result = listen_fn.call2(&JsValue::UNDEFINED, &event_name, &handler);
                    info!("[ORIGA-UI] listen() call result: {:?}", result);
                    callback.forget();
                    info!("[ORIGA-UI] Tauri deep-link listener registered successfully");
                    return;
                }
            }
        }
    }

    info!("[ORIGA-UI] Not in Tauri environment or failed to register listener");
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
                                    Ok(user) => {
                                        ctx_clone.current_user.set(Some(user));
                                        if let Some(window) = web_sys::window()
                                            && let Err(e) = window.location().set_href("/home")
                                        {
                                            error!("Failed to redirect to /home: {:?}", e);
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to create profile: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
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
    provide_context(auth_context.current_user);
    provide_context(auth_context.clone());

    check_url_oauth_callback(&auth_context);
    setup_oauth_listener(auth_context.clone());

    let ctx = auth_context.clone();
    spawn_local(async move {
        ctx.init_dictionary().await;
    });

    let ctx = auth_context.clone();
    spawn_local(async move {
        ctx.init_session().await;
    });

    let loading_message = Signal::derive(move || {
        let session = auth_context.is_session_loading.get();
        let dict = auth_context.is_dictionary_loading.get();

        if session && dict {
            "Проверка авторизации и загрузка словаря...".to_string()
        } else if session {
            "Проверка авторизации...".to_string()
        } else if dict {
            "Загрузка словаря Unidic (при первом запуске может занять несколько минут)..."
                .to_string()
        } else {
            "".to_string()
        }
    });

    view! {
        <Show when=move || auth_context.is_session_loading.get() || auth_context.is_dictionary_loading.get()>
            <LoadingOverlay message=loading_message />
        </Show>
        <AppRoutes />
    }
}
