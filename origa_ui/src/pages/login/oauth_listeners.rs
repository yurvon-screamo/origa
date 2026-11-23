use crate::core::tauri;
use crate::i18n::{I18nContext, Locale};
use crate::pages::login::auth_handlers::{handle_oauth_callback, handle_oauth_callback_desktop};
use crate::repository::{OAuthFailure, take_pkce_verifier_async};
use crate::store::auth_store::AuthStore;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::prelude::*;
use origa::domain::User;
use std::sync::{Mutex, OnceLock};
use tracing::{debug, error, trace, warn};
use wasm_bindgen_futures::JsFuture;

const LOGIN_PATH: &str = "/login";

/// The deep-link URL prefix this listener owns (the ownership contract is
/// documented in `core/shortcut_links.rs`; shortcut `origa://add*` and
/// selftest `origa://selftest-ocr` URLs are routed by their own listeners).
/// Prefix match is intentional: URLs arrive from our own deep-link
/// transport, and all three call sites (warm event, cold poll, resume poll)
/// must share identical semantics.
fn is_auth_callback_url(url: &str) -> bool {
    url.starts_with("origa://auth/callback")
}

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

        // Routing foreign URLs out here keeps them silent: no oauth_error,
        // no is_oauth_loading flicker on /login, no Sentry noise.
        if !is_auth_callback_url(&url) {
            debug!(url = %url, "non-auth deep-link — skipping oauth processing");
            return;
        }

        let auth_store = auth_store_clone.clone();
        let i18n = i18n;
        auth_store.oauth_error.set(None);
        spawn_local(async move {
            debug!(url = %url, "processing oauth url");
            *last_processed_callback()
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = Some(url.clone());
            auth_store.is_oauth_loading.set(true);
            let result = process_oauth_url(&url, &auth_store, &i18n).await;
            debug!(result = ?result, "process_oauth_url result");
            handle_oauth_result(result, &auth_store);
            auth_store.is_oauth_loading.set(false);
        });
    });

    register_tauri_listener(callback);

    poll_current_deep_link(auth_store, i18n);
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

pub(crate) async fn process_oauth_url(
    url: &str,
    auth_store: &AuthStore,
    i18n: &I18nContext<Locale>,
) -> Result<Option<User>, String> {
    let parsed = url::Url::parse(url);
    trace!(parsed = ?parsed, "URL parse result");

    if parsed.is_ok_and(|u| u.query_pairs().any(|(k, _)| k == "code")) {
        debug!("URL has 'code' param, calling handle_oauth_callback_desktop");
        return handle_oauth_callback_desktop(url, auth_store, i18n).await;
    }

    if let Some(fragment) = url.split('#').nth(1) {
        debug!(fragment = %fragment, "URL has fragment, calling handle_oauth_callback");
        return handle_oauth_callback(fragment, auth_store, i18n)
            .await
            .map(Some);
    }

    error!(url = %url, "URL has no 'code' param and no fragment");
    Err(i18n
        .get_keys_untracked()
        .login()
        .unsupported_callback()
        .inner()
        .to_string())
}

pub(crate) fn handle_oauth_result(result: Result<Option<User>, String>, auth_store: &AuthStore) {
    match result {
        Ok(Some(user)) => {
            debug!(user_id = %user.id(), "OAuth success — updating user signal");
            auth_store.user.set(Some(user));
        },
        Ok(None) => {
            debug!("OAuth callback skipped (verifier already consumed)");
        },
        Err(e) => {
            error!(error = %e, "OAuth callback error");
            auth_store.oauth_error.set(Some(e));
        },
    }
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

fn poll_current_deep_link(auth_store: AuthStore, i18n: I18nContext<Locale>) {
    debug!("poll_current_deep_link() called");

    // Optimization: if a session is already restored, there is nothing to do.
    // This is not the correctness guarantee (check_session may not have
    // restored the user yet on a fresh mount) — the verifier-existence skip in
    // handle_oauth_callback_desktop is the guarantee against replayed callbacks.
    if auth_store.is_authenticated().get_untracked() {
        debug!("already authenticated — skipping current deep-link poll");
        return;
    }

    let Some(invoke_fn) = tauri::invoke_fn() else {
        debug!("__TAURI__.core.invoke not available — not in Tauri, skipping");
        return;
    };

    let Ok(result) = invoke_fn.call1(
        &JsValue::UNDEFINED,
        &JsValue::from_str("get_current_deep_link"),
    ) else {
        warn!("get_current_deep_link invoke call failed");
        return;
    };
    let Ok(promise) = result.dyn_into::<js_sys::Promise>() else {
        warn!("get_current_deep_link did not return a Promise");
        return;
    };

    let auth_store_clone = auth_store.clone();
    let i18n_clone = i18n;

    let on_resolve = Closure::<dyn FnMut(JsValue)>::new(move |value: JsValue| {
        let url = value.as_string().unwrap_or_default();
        if url.is_empty() {
            debug!("no current deep-link");
            return;
        }
        if !is_auth_callback_url(&url) {
            debug!(url = %url, "current deep-link is not an auth callback — skipping");
            return;
        }
        debug!(url = %url, "processing current deep-link from app load");

        let auth_store = auth_store_clone.clone();
        let i18n = i18n_clone;
        auth_store.oauth_error.set(None);
        spawn_local(async move {
            *last_processed_callback()
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = Some(url.clone());
            auth_store.is_oauth_loading.set(true);
            let result = process_oauth_url(&url, &auth_store, &i18n).await;
            debug!(result = ?result, "current deep-link process result");
            handle_oauth_result(result, &auth_store);
            auth_store.is_oauth_loading.set(false);
        });
    });

    let on_reject = Closure::<dyn FnMut(JsValue)>::new(|err: JsValue| {
        warn!(?err, "get_current_deep_link promise rejected");
    });

    let _ = promise.then2(&on_resolve, &on_reject);
    on_resolve.forget();
    on_reject.forget();
}

async fn fetch_current_deep_link_url() -> Option<String> {
    let invoke_fn = tauri::invoke_fn()?;
    let result = invoke_fn
        .call1(
            &JsValue::UNDEFINED,
            &JsValue::from_str("get_current_deep_link"),
        )
        .ok()?;
    let promise = result.dyn_into::<js_sys::Promise>().ok()?;
    JsFuture::from(promise).await.ok()?.as_string()
}

// Cross-attempt memory of the last auth-callback URL processed by the poll.
// `get_current()` does not clear after read (returns the same URL until a new
// deep link arrives), so without this a retry that starts a new poll would
// re-process the stale previous URL and stop before the fresh redirect arrives.
static LAST_PROCESSED_AUTH_CALLBACK: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn last_processed_callback() -> &'static Mutex<Option<String>> {
    LAST_PROCESSED_AUTH_CALLBACK.get_or_init(|| Mutex::new(None))
}

/// Polls the pending deep-link URL on a short interval after the user opens the
/// OAuth provider externally. On Android the WebView JS is frozen while
/// backgrounded, so the live `deep-link-received` event emitted during that
/// window is lost; the poll's timer is paused while frozen and resumes on
/// Activity `onResume`, recovering the pending callback URL. See ADR-010.
pub fn start_resume_polling(auth_store: AuthStore, i18n: I18nContext<Locale>) {
    spawn_local(async move {
        const POLL_INTERVAL_MS: u32 = 1500;
        const MAX_POLLS: usize = 120;

        for _ in 0..MAX_POLLS {
            TimeoutFuture::new(POLL_INTERVAL_MS).await;

            if auth_store.user.with(|u| u.is_some()) {
                debug!("resume-poll: authenticated, stopping");
                return;
            }

            let Some(url) = fetch_current_deep_link_url().await else {
                continue;
            };
            if !is_auth_callback_url(&url) {
                continue;
            }

            let already_processed = last_processed_callback()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_deref()
                == Some(url.as_str());
            if already_processed {
                // Stale URL from a previous attempt; keep polling for a fresh one.
                continue;
            }

            debug!(url = %url, "resume-poll: processing auth callback");
            *last_processed_callback()
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = Some(url.clone());
            auth_store.oauth_error.set(None);
            auth_store.is_oauth_loading.set(true);
            let result = process_oauth_url(&url, &auth_store, &i18n).await;
            debug!(result = ?result, "resume-poll process result");
            handle_oauth_result(result, &auth_store);
            auth_store.is_oauth_loading.set(false);
            return;
        }
        debug!("resume-poll: deadline reached without callback");
    });
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
            auth_store.oauth_error.set(Some(match e {
                OAuthFailure::Network => i18n
                    .get_keys_untracked()
                    .login()
                    .login_retry_error()
                    .inner()
                    .to_string(),
                OAuthFailure::Message(message) => message,
            }));
        },
    }
}

#[cfg(test)]
mod auth_callback_routing_tests {
    use super::is_auth_callback_url;

    /// The oauth listener owns `origa://auth/callback*` (ownership contract
    /// documented in `core/shortcut_links.rs`): every deep-link URL that
    /// reaches `process_oauth_url` must be an auth-callback URL. Foreign
    /// prefixes (shortcut `origa://add*`, selftest `origa://selftest-ocr`)
    /// are skipped silently by the routing filter before any auth state is
    /// touched — a foreign URL must not produce an `oauth_error` or flicker
    /// `is_oauth_loading` on /login.
    #[rstest::rstest]
    #[case::code_param("origa://auth/callback?code=abc", true)]
    #[case::fragment("origa://auth/callback#access_token=x", true)]
    #[case::bare_callback("origa://auth/callback", true)]
    #[case::selftest_probe("origa://selftest-ocr", false)]
    #[case::shortcut("origa://add?source=camera", false)]
    #[case::web_login_url("https://app.origa.uwuwu.net/login?code=x", false)]
    #[case::empty("", false)]
    #[case::case_sensitive("ORIGA://AUTH/CALLBACK?code=x", false)]
    fn routes_auth_callback_urls(#[case] url: &str, #[case] expected: bool) {
        assert_eq!(is_auth_callback_url(url), expected, "routing of {url}");
    }
}
