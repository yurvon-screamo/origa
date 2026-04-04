use crate::pages::login::auth_handlers::{handle_oauth_callback, handle_oauth_callback_desktop};
use crate::store::auth_store::AuthStore;
use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::prelude::*;
use tracing::error;
use web_sys::console;

const LOGIN_PATH: &str = "/login";
const HOME_PATH: &str = "/home";

pub fn setup_oauth_listener(auth_store: AuthStore) {
    console::log_1(&"[oauth-listener] setup_oauth_listener() called".into());

    let auth_store_clone = auth_store.clone();
    let callback = Closure::<dyn Fn(JsValue)>::new(move |event: JsValue| {
        console::log_1(&"[oauth-listener] deep-link-received event fired!".into());
        console::log_1(&format!("[oauth-listener] raw event: {:?}", event).into());

        let url = extract_url_from_event(&event);
        console::log_1(&format!("[oauth-listener] extracted url: '{}'", url).into());

        if url.is_empty() {
            console::warn_1(&"[oauth-listener] url is empty, ignoring event".into());
            return;
        }

        let auth_store = auth_store_clone.clone();
        auth_store.oauth_error.set(None);
        spawn_local(async move {
            console::log_1(&format!("[oauth-listener] processing url: {}", url).into());
            let result = process_oauth_url(&url, &auth_store).await;
            console::log_1(
                &format!("[oauth-listener] process_oauth_url result: {:?}", result).into(),
            );
            handle_oauth_result(result, &auth_store);
        });
    });

    register_tauri_listener(callback);
}

fn extract_url_from_event(event: &JsValue) -> String {
    if let Ok(url_str) = js_sys::Reflect::get(event, &JsValue::from_str("payload"))
        && let Some(s) = url_str.as_string()
    {
        console::log_1(&format!("[oauth-listener] extracted from event.payload: {}", s).into());
        return s;
    }

    if let Some(s) = event.as_string() {
        console::log_1(&format!("[oauth-listener] extracted from event.as_string: {}", s).into());
        return s;
    }

    error!(
        "[oauth-listener] Invalid deep-link event format: {:?}",
        event
    );
    String::new()
}

async fn process_oauth_url(url: &str, auth_store: &AuthStore) -> Result<(), String> {
    let parsed = url::Url::parse(url);
    console::log_1(&format!("[oauth-listener] URL parse result: {:?}", parsed).into());

    if parsed.is_ok_and(|u| u.query_pairs().any(|(k, _)| k == "code")) {
        console::log_1(
            &"[oauth-listener] URL has 'code' param, calling handle_oauth_callback_desktop".into(),
        );
        handle_oauth_callback_desktop(url, auth_store).await?;
        return Ok(());
    }

    if let Some(fragment) = url.split('#').nth(1) {
        console::log_1(
            &format!(
                "[oauth-listener] URL has fragment, calling handle_oauth_callback: {}",
                fragment
            )
            .into(),
        );
        handle_oauth_callback(fragment, auth_store).await?;
        return Ok(());
    }

    console::error_1(
        &format!(
            "[oauth-listener] URL has no 'code' param and no fragment: {}",
            url
        )
        .into(),
    );
    Err("Неподдерживаемый формат callback URL".to_string())
}

fn handle_oauth_result(result: Result<(), String>, auth_store: &AuthStore) {
    if let Err(e) = result {
        error!("[oauth-listener] OAuth callback error: {}", e);
        console::error_1(&format!("[oauth-listener] OAuth error: {}", e).into());
        auth_store.oauth_error.set(Some(e));
        return;
    }

    console::log_1(&"[oauth-listener] OAuth success, redirecting to /home".into());

    if let Some(window) = web_sys::window()
        && let Err(e) = window.location().set_href(HOME_PATH)
    {
        error!(
            "[oauth-listener] Failed to redirect to {}: {:?}",
            HOME_PATH, e
        );
    }
}

fn register_tauri_listener(callback: Closure<dyn Fn(JsValue)>) {
    console::log_1(&"[oauth-listener] register_tauri_listener() called".into());

    let Some(window) = web_sys::window() else {
        console::error_1(&"[oauth-listener] no window object".into());
        return;
    };

    let Some(tauri_obj) = get_tauri_object(&window) else {
        console::warn_1(
            &"[oauth-listener] __TAURI__ not found on window — not in Tauri desktop, skipping"
                .into(),
        );
        return;
    };
    console::log_1(&"[oauth-listener] __TAURI__ found".into());

    let Some(event_mod) = get_event_module(tauri_obj.as_ref()) else {
        console::error_1(&"[oauth-listener] __TAURI__.event not found".into());
        return;
    };
    console::log_1(&"[oauth-listener] __TAURI__.event found".into());

    let Some(listen_fn) = get_listen_function(&event_mod) else {
        console::error_1(&"[oauth-listener] __TAURI__.event.listen not found".into());
        return;
    };
    console::log_1(&"[oauth-listener] __TAURI__.event.listen function found".into());

    let event_name = JsValue::from_str("deep-link-received");
    let handler = callback.as_ref().clone();

    let result = listen_fn.call2(&JsValue::UNDEFINED, &event_name, &handler);
    console::log_1(&format!("[oauth-listener] listen() call result: {:?}", result).into());

    if result.is_ok() {
        console::log_1(
            &"[oauth-listener] listener registered successfully, forgetting callback".into(),
        );
        callback.forget();
    } else {
        console::error_1(&"[oauth-listener] listen() call FAILED".into());
    }
}

fn get_tauri_object(window: &web_sys::Window) -> Option<js_sys::Object> {
    js_sys::Reflect::get(window, &JsValue::from_str("__TAURI__"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Object>().ok())
}

fn get_event_module(tauri_obj: &JsValue) -> Option<JsValue> {
    js_sys::Reflect::get(tauri_obj, &JsValue::from_str("event")).ok()
}

fn get_listen_function(event_mod: &JsValue) -> Option<js_sys::Function> {
    js_sys::Reflect::get(event_mod, &JsValue::from_str("listen"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
}

pub fn check_url_oauth_callback(auth_store: &AuthStore) {
    let disposed = StoredValue::new(());
    let path = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();

    if path != LOGIN_PATH {
        return;
    }

    let search = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default();

    let Some(code) = extract_oauth_code(&search) else {
        return;
    };

    let verifier = get_and_delete_verifier();
    if verifier.is_none() {
        return;
    }

    let is_oauth_loading = auth_store.is_oauth_loading;
    is_oauth_loading.set(true);
    auth_store.oauth_error.set(None);
    let auth_store_clone = auth_store.clone();

    spawn_local(async move {
        process_oauth_flow(
            auth_store_clone,
            verifier.unwrap(),
            code,
            is_oauth_loading,
            disposed,
        )
        .await;
    });
}

fn extract_oauth_code(search: &str) -> Option<String> {
    search
        .strip_prefix("?code=")
        .map(|code| code.split('&').next().unwrap_or(code).to_string())
}

fn get_and_delete_verifier() -> Option<String> {
    let verifier: Option<String> = LocalStorage::get("pkce_verifier").ok();
    LocalStorage::delete("pkce_verifier");
    verifier
}

async fn process_oauth_flow(
    auth_store: AuthStore,
    verifier: String,
    code: String,
    is_oauth_loading: RwSignal<bool>,
    disposed: StoredValue<()>,
) {
    let result = auth_store.set_oauth_session(&code, &verifier).await;

    if disposed.is_disposed() {
        return;
    }

    match result {
        Ok(_) => {
            redirect_to_home();
        },
        Err(e) => {
            is_oauth_loading.set(false);
            error!("OAuth flow failed: {:?}", e);
            auth_store.oauth_error.set(Some(e.to_string()));
        },
    }
}

fn redirect_to_home() {
    if let Some(window) = web_sys::window()
        && let Err(e) = window.location().set_href(HOME_PATH)
    {
        error!("Failed to redirect to {}: {:?}", HOME_PATH, e);
    }
}
