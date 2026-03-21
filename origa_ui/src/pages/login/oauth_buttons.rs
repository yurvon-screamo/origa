use crate::repository::OAuthProvider;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;

#[component]
pub fn OAuthButtons() -> impl IntoView {
    view! {
        <div class="space-y-3">
            <button
                type="button"
                class="w-full flex items-center justify-center gap-3 px-4 py-3 border border-[var(--border-dark)] rounded-lg bg-[var(--bg-cream)] hover:bg-[var(--bg-aged)] transition-colors"
                on:click=move |_: leptos::ev::MouseEvent| {
                    open_oauth_url(OAuthProvider::Google);
                }
            >
                <GoogleIcon />
                <span class="text-[var(--fg-black)]">"Войти через Google"</span>
            </button>

            <button
                type="button"
                class="w-full flex items-center justify-center gap-3 px-4 py-3 border border-[var(--border-dark)] rounded-lg bg-[var(--bg-cream)] hover:bg-[var(--bg-aged)] transition-colors"
                on:click=move |_: leptos::ev::MouseEvent| {
                    open_oauth_url(OAuthProvider::Yandex);
                }
            >
                <YandexIcon />
                <span class="text-[var(--fg-black)]">"Войти через Yandex"</span>
            </button>
        </div>
    }
}

fn is_tauri_desktop() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    js_sys::Reflect::get(&window, &JsValue::from_str("isTauri"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn open_url_external(url: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };

    if !is_tauri_desktop() {
        let _ = window.location().set_href(url);
        return;
    }

    let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")) else {
        let _ = window.open_with_url_and_target(url, "_blank");
        return;
    };

    let Ok(opener) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("opener")) else {
        let _ = window.open_with_url_and_target(url, "_blank");
        return;
    };

    let Ok(open_url_fn) = js_sys::Reflect::get(&opener, &JsValue::from_str("openUrl")) else {
        let _ = window.open_with_url_and_target(url, "_blank");
        return;
    };

    let Ok(open_url_fn) = open_url_fn.dyn_into::<js_sys::Function>() else {
        let _ = window.open_with_url_and_target(url, "_blank");
        return;
    };

    if open_url_fn
        .call1(&JsValue::UNDEFINED, &JsValue::from_str(url))
        .is_err()
    {
        let _ = window.open_with_url_and_target(url, "_blank");
    }
}

fn open_oauth_url(provider: OAuthProvider) {
    use crate::repository::TrailBaseClient;
    use crate::repository::trailbase_auth::{generate_pkce_challenge, generate_pkce_verifier};
    use gloo_storage::{LocalStorage, Storage};
    use web_sys::console;

    let redirect_uri = if is_tauri_desktop() {
        "https://origa.uwuwu.net/public/auth/desktop-callback.html".to_string()
    } else {
        let window = web_sys::window().expect("window not available");
        let base_url = window.location().origin().unwrap_or_default();
        format!("{}/login", base_url)
    };

    console::log_1(&JsValue::from_str(&format!(
        "Redirect URI: {}",
        redirect_uri
    )));

    let verifier = generate_pkce_verifier();
    console::log_1(&JsValue::from_str(&format!(
        "Generated PKCE verifier: {}",
        verifier
    )));

    let challenge = generate_pkce_challenge(&verifier);

    LocalStorage::set("pkce_verifier", &verifier).ok();
    console::log_1(&JsValue::from_str("Saved verifier to LocalStorage"));

    let client = TrailBaseClient::new();
    let url = client.get_oauth_url(provider.as_str(), &redirect_uri, &challenge);

    open_url_external(&url);
}

#[component]
fn GoogleIcon() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path
                fill="#4285F4"
                d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
            />
            <path
                fill="#34A853"
                d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
            />
            <path
                fill="#FBBC05"
                d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
            />
            <path
                fill="#EA4335"
                d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
            />
        </svg>
    }
}

#[component]
fn YandexIcon() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path fill="#FC3F1D" d="M3 3h18v18H3V3z" />
            <path
                fill="var(--bg-paper)"
                d="M13.32 18.82V12.7l2.94-7.52h-2.66l-1.57 4.38-1.57-4.38H7.8l2.94 7.52v6.12h2.58z"
            />
        </svg>
    }
}
