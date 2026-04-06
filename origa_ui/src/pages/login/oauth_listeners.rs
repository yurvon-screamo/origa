use crate::pages::login::auth_handlers::{handle_oauth_callback, handle_oauth_callback_desktop};
use crate::store::auth_store::AuthStore;
use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::prelude::*;
use tracing::{debug, error, trace, warn};

const LOGIN_PATH: &str = "/login";
const HOME_PATH: &str = "/home";

pub fn setup_oauth_listener(auth_store: AuthStore) {
    debug!("setup_oauth_listener() called");

    let auth_store_clone = auth_store.clone();
    let callback = Closure::<dyn Fn(JsValue)>::new(move |event: JsValue| {
        debug!("deep-link-received event fired");
        trace!(event = ?event, "raw event");

        let url = extract_url_from_event(&event);
        debug!(url = %url, "extracted url from event");

        if url.is_empty() {
            warn!("url is empty, ignoring event");
            return;
        }

        let auth_store = auth_store_clone.clone();
        auth_store.oauth_error.set(None);
        spawn_local(async move {
            debug!(url = %url, "processing oauth url");
            let result = process_oauth_url(&url, &auth_store).await;
            debug!(result = ?result, "process_oauth_url result");
            handle_oauth_result(result, &auth_store);
        });
    });

    register_tauri_listener(callback);
}

fn extract_url_from_event(event: &JsValue) -> String {
    if let Ok(url_str) = js_sys::Reflect::get(event, &JsValue::from_str("payload"))
        && let Some(s) = url_str.as_string()
    {
        trace!(url = %s, "extracted from event.payload");
        return s;
    }

    if let Some(s) = event.as_string() {
        trace!(url = %s, "extracted from event.as_string");
        return s;
    }

    error!(event = ?event, "invalid deep-link event format");
    String::new()
}

async fn process_oauth_url(url: &str, auth_store: &AuthStore) -> Result<(), String> {
    let parsed = url::Url::parse(url);
    trace!(parsed = ?parsed, "URL parse result");

    if parsed.is_ok_and(|u| u.query_pairs().any(|(k, _)| k == "code")) {
        debug!("URL has 'code' param, calling handle_oauth_callback_desktop");
        handle_oauth_callback_desktop(url, auth_store).await?;
        return Ok(());
    }

    if let Some(fragment) = url.split('#').nth(1) {
        debug!(fragment = %fragment, "URL has fragment, calling handle_oauth_callback");
        handle_oauth_callback(fragment, auth_store).await?;
        return Ok(());
    }

    error!(url = %url, "URL has no 'code' param and no fragment");
    Err("Неподдерживаемый формат callback URL".to_string())
}

fn handle_oauth_result(result: Result<(), String>, auth_store: &AuthStore) {
    if let Err(e) = result {
        error!(error = %e, "OAuth callback error");
        auth_store.oauth_error.set(Some(e));
        return;
    }

    debug!("OAuth success, redirecting to /home");

    if let Some(window) = web_sys::window()
        && let Err(e) = window.location().set_href(HOME_PATH)
    {
        error!(path = HOME_PATH, error = ?e, "failed to redirect");
    }
}

fn register_tauri_listener(callback: Closure<dyn Fn(JsValue)>) {
    debug!("register_tauri_listener() called");

    let Some(window) = web_sys::window() else {
        error!("no window object");
        return;
    };

    let Some(tauri_obj) = get_tauri_object(&window) else {
        warn!("__TAURI__ not found on window — not in Tauri desktop, skipping");
        return;
    };
    debug!("__TAURI__ found on window");

    let Some(event_mod) = get_event_module(tauri_obj.as_ref()) else {
        error!("__TAURI__.event not found");
        return;
    };
    debug!("__TAURI__.event found");

    let Some(listen_fn) = get_listen_function(&event_mod) else {
        error!("__TAURI__.event.listen not found");
        return;
    };
    debug!("__TAURI__.event.listen function found");

    let event_name = JsValue::from_str("deep-link-received");
    let handler = callback.as_ref().clone();

    let result = listen_fn.call2(&JsValue::UNDEFINED, &event_name, &handler);
    trace!(result = ?result, "listen() call result");

    if result.is_ok() {
        debug!("listener registered successfully, forgetting callback");
        callback.forget();
    } else {
        error!("listen() call failed");
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
            error!(?e, "OAuth flow failed");
            auth_store.oauth_error.set(Some(e.to_string()));
        },
    }
}

fn redirect_to_home() {
    if let Some(window) = web_sys::window()
        && let Err(e) = window.location().set_href(HOME_PATH)
    {
        error!(path = HOME_PATH, ?e, "failed to redirect");
    }
}
