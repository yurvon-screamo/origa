use crate::core::tauri;
use crate::i18n::{I18nContext, Locale};
use crate::pages::login::auth_handlers::{handle_oauth_callback, handle_oauth_callback_desktop};
use crate::repository::take_pkce_verifier_async;
use crate::store::auth_store::AuthStore;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::prelude::*;
use tracing::{debug, error, trace, warn};

const LOGIN_PATH: &str = "/login";

pub fn setup_oauth_listener(auth_store: AuthStore, i18n: I18nContext<Locale>) {
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
        let i18n = i18n;
        auth_store.oauth_error.set(None);
        spawn_local(async move {
            debug!(url = %url, "processing oauth url");
            let result = process_oauth_url(&url, &auth_store, &i18n).await;
            debug!(result = ?result, "process_oauth_url result");
            handle_oauth_result(result, &auth_store);
        });
    });

    register_tauri_listener(callback);

    check_pending_deep_link(auth_store, i18n);
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

async fn process_oauth_url(
    url: &str,
    auth_store: &AuthStore,
    i18n: &I18nContext<Locale>,
) -> Result<(), String> {
    let parsed = url::Url::parse(url);
    trace!(parsed = ?parsed, "URL parse result");

    if parsed.is_ok_and(|u| u.query_pairs().any(|(k, _)| k == "code")) {
        debug!("URL has 'code' param, calling handle_oauth_callback_desktop");
        handle_oauth_callback_desktop(url, auth_store, i18n).await?;
        return Ok(());
    }

    if let Some(fragment) = url.split('#').nth(1) {
        debug!(fragment = %fragment, "URL has fragment, calling handle_oauth_callback");
        handle_oauth_callback(fragment, auth_store, i18n).await?;
        return Ok(());
    }

    error!(url = %url, "URL has no 'code' param and no fragment");
    Err(i18n
        .get_keys_untracked()
        .login()
        .unsupported_callback()
        .inner()
        .to_string())
}

fn handle_oauth_result(result: Result<(), String>, auth_store: &AuthStore) {
    if let Err(e) = result {
        error!(error = %e, "OAuth callback error");
        auth_store.oauth_error.set(Some(e));
        return;
    }

    debug!("OAuth success — navigation to /home handled by App() Effect");
}

fn register_tauri_listener(callback: Closure<dyn Fn(JsValue)>) {
    debug!("register_tauri_listener() called");

    let Some(listen_fn) = tauri::event_listen_fn() else {
        warn!("__TAURI__.event.listen not available — not in Tauri, skipping");
        return;
    };
    debug!("__TAURI__.event.listen found");

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

fn check_pending_deep_link(auth_store: AuthStore, i18n: I18nContext<Locale>) {
    debug!("check_pending_deep_link() called");

    let Some(invoke_fn) = tauri::invoke_fn() else {
        debug!("__TAURI__.core.invoke not available — not in Tauri, skipping");
        return;
    };

    let Ok(result) = invoke_fn.call1(
        &JsValue::UNDEFINED,
        &JsValue::from_str("get_pending_deep_link"),
    ) else {
        warn!("get_pending_deep_link invoke call failed");
        return;
    };
    let Ok(promise) = result.dyn_into::<js_sys::Promise>() else {
        warn!("get_pending_deep_link did not return a Promise");
        return;
    };

    let auth_store_clone = auth_store.clone();
    let i18n_clone = i18n;

    let on_resolve = Closure::<dyn FnMut(JsValue)>::new(move |value: JsValue| {
        let url = value.as_string().unwrap_or_default();
        if url.is_empty() {
            debug!("no pending deep-link");
            return;
        }
        debug!(url = %url, "processing pending deep-link from cold start");

        let auth_store = auth_store_clone.clone();
        let i18n = i18n_clone;
        auth_store.oauth_error.set(None);
        spawn_local(async move {
            let result = process_oauth_url(&url, &auth_store, &i18n).await;
            debug!(result = ?result, "pending deep-link process result");
            handle_oauth_result(result, &auth_store);
        });
    });

    let on_reject = Closure::<dyn FnMut(JsValue)>::new(|err: JsValue| {
        warn!(?err, "get_pending_deep_link promise rejected");
    });

    let _ = promise.then2(&on_resolve, &on_reject);
    on_resolve.forget();
    on_reject.forget();
}

pub fn check_url_oauth_callback(auth_store: &AuthStore, i18n: &I18nContext<Locale>) {
    let i18n = *i18n;
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

    let is_oauth_loading = auth_store.is_oauth_loading;
    is_oauth_loading.set(true);
    auth_store.oauth_error.set(None);
    let auth_store_clone = auth_store.clone();

    spawn_local(async move {
        let Some(verifier) = take_pkce_verifier_async().await else {
            is_oauth_loading.set(false);
            warn!("PKCE verifier not found for OAuth callback");
            return;
        };

        process_oauth_flow(
            auth_store_clone,
            verifier,
            code,
            is_oauth_loading,
            disposed,
            i18n,
        )
        .await;
    });
}

fn extract_oauth_code(search: &str) -> Option<String> {
    search
        .strip_prefix("?code=")
        .map(|code| code.split('&').next().unwrap_or(code).to_string())
}

async fn process_oauth_flow(
    auth_store: AuthStore,
    verifier: String,
    code: String,
    is_oauth_loading: RwSignal<bool>,
    disposed: StoredValue<()>,
    i18n: I18nContext<Locale>,
) {
    let result = auth_store.set_oauth_session(&code, &verifier, &i18n).await;

    if disposed.is_disposed() {
        return;
    }

    match result {
        Ok(_) => {
            debug!("OAuth flow success — navigation to /home handled by App() Effect");
        },
        Err(e) => {
            is_oauth_loading.set(false);
            error!(?e, "OAuth flow failed");
            auth_store.oauth_error.set(Some(e.to_string()));
        },
    }
}
