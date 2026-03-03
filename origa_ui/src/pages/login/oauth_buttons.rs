use crate::repository::{OAuthProvider, SupabaseClient};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsValue;

#[component]
pub fn OAuthButtons() -> impl IntoView {
    view! {
        <div class="space-y-3">
            <button
                type="button"
                class="w-full flex items-center justify-center gap-3 px-4 py-3 border border-[var(--border-color)] rounded-lg bg-[var(--bg-primary)] hover:bg-[var(--bg-secondary)] transition-colors"
                on:click=move |_: leptos::ev::MouseEvent| {
                    open_oauth_url(OAuthProvider::Google);
                }
            >
                <GoogleIcon />
                <span class="text-[var(--fg)]">"Войти через Google"</span>
            </button>

            <button
                type="button"
                class="w-full flex items-center justify-center gap-3 px-4 py-3 border border-[var(--border-color)] rounded-lg bg-[var(--bg-primary)] hover:bg-[var(--bg-secondary)] transition-colors"
                on:click=move |_: leptos::ev::MouseEvent| {
                    open_oauth_url(OAuthProvider::Yandex);
                }
            >
                <YandexIcon />
                <span class="text-[var(--fg)]">"Войти через Yandex"</span>
            </button>
        </div>
    }
}

fn open_oauth_url(provider: OAuthProvider) {
    let window = web_sys::window().expect("window not available");

    // Check if running in Tauri (has __TAURI__ AND is not http/https protocol)
    let is_tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")).is_ok()
        && window.location().protocol().unwrap_or_default() == "tauri:";

    let url = if is_tauri {
        web_sys::console::log_1(&"OAuth: Tauri mode".into());
        SupabaseClient::get_oauth_url(provider.as_str())
    } else {
        let base_url = window.location().origin().unwrap_or_default();
        let redirect_uri = format!("{}/login", base_url);
        web_sys::console::log_1(&format!("OAuth: Web mode, redirect_uri={}", redirect_uri).into());
        let url = SupabaseClient::get_oauth_url_with_redirect(provider.as_str(), &redirect_uri);
        web_sys::console::log_1(&format!("OAuth: Generated URL={}", url).into());
        url
    };

    let _ = window.location().set_href(&url);
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
                fill="#fff"
                d="M13.32 18.82V12.7l2.94-7.52h-2.66l-1.57 4.38-1.57-4.38H7.8l2.94 7.52v6.12h2.58z"
            />
        </svg>
    }
}
