use crate::pages::login::auth_handlers::{handle_oauth_callback, handle_oauth_callback_desktop};
use crate::store::auth_store::AuthStore;
use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::prelude::*;
use tracing::error;

const LOGIN_PATH: &str = "/login";
const HOME_PATH: &str = "/home";

pub fn setup_oauth_listener(auth_store: AuthStore) {
    let auth_store_clone = auth_store.clone();
    let callback = Closure::<dyn Fn(JsValue)>::new(move |event: JsValue| {
        let url = extract_url_from_event(&event);
        if url.is_empty() {
            return;
        }

        let auth_store = auth_store_clone.clone();
        spawn_local(async move {
            let result = process_oauth_url(&url, &auth_store).await;
            handle_oauth_result(result, &auth_store);
        });
    });

    register_tauri_listener(callback);
}

fn extract_url_from_event(event: &JsValue) -> String {
    if let Ok(url_str) = js_sys::Reflect::get(event, &JsValue::from_str("payload"))
        && let Some(s) = url_str.as_string()
    {
        return s;
    }

    if let Some(s) = event.as_string() {
        return s;
    }

    error!("Invalid deep-link event format: {:?}", event);
    String::new()
}

async fn process_oauth_url(url: &str, auth_store: &AuthStore) -> Result<(), String> {
    if url::Url::parse(url).is_ok_and(|u| u.query_pairs().any(|(k, _)| k == "code")) {
        handle_oauth_callback_desktop(url, auth_store).await?;
        return Ok(());
    }

    if let Some(fragment) = url.split('#').nth(1) {
        handle_oauth_callback(fragment, auth_store).await?;
        return Ok(());
    }

    Err("Неподдерживаемый формат callback URL".to_string())
}

fn handle_oauth_result(result: Result<(), String>, _auth_store: &AuthStore) {
    if let Err(e) = result {
        error!("OAuth callback error: {}", e);
        return;
    }

    if let Some(window) = web_sys::window()
        && let Err(e) = window.location().set_href(HOME_PATH)
    {
        error!("Failed to redirect to {}: {:?}", HOME_PATH, e);
    }
}

fn register_tauri_listener(callback: Closure<dyn Fn(JsValue)>) {
    let Some(window) = web_sys::window() else {
        return;
    };

    let Some(tauri_obj) = get_tauri_object(&window) else {
        return;
    };

    let Some(event_mod) = get_event_module(&tauri_obj) else {
        return;
    };

    let Some(listen_fn) = get_listen_function(&event_mod) else {
        return;
    };

    let event_name = JsValue::from_str("deep-link-received");
    let handler = callback.as_ref().clone();

    if listen_fn
        .call2(&JsValue::UNDEFINED, &event_name, &handler)
        .is_ok()
    {
        // forget() корректен только при успешной регистрации
        callback.forget();
    }
}

fn get_tauri_object(window: &web_sys::Window) -> Option<js_sys::Object> {
    js_sys::Reflect::get(window, &JsValue::from_str("__TAURI__"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Object>().ok())
}

fn get_event_module(tauri_obj: &js_sys::Object) -> Option<js_sys::Object> {
    js_sys::Reflect::get(tauri_obj, &JsValue::from_str("event"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Object>().ok())
}

fn get_listen_function(event_mod: &js_sys::Object) -> Option<js_sys::Function> {
    js_sys::Reflect::get(event_mod, &JsValue::from_str("listen"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
}

pub fn check_url_oauth_callback(auth_store: &AuthStore) {
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
    let auth_store_clone = auth_store.clone();

    spawn_local(async move {
        process_oauth_flow(auth_store_clone, verifier.unwrap(), code, is_oauth_loading).await;
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
) {
    let result = auth_store.set_oauth_session(&code, &verifier).await;

    match result {
        Ok(_) => {
            redirect_to_home();
        }
        Err(e) => {
            is_oauth_loading.set(false);
            error!("OAuth flow failed: {:?}", e);
        }
    }
}

fn redirect_to_home() {
    if let Some(window) = web_sys::window()
        && let Err(e) = window.location().set_href(HOME_PATH)
    {
        error!("Failed to redirect to {}: {:?}", HOME_PATH, e);
    }
}
