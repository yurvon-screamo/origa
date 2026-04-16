use leptos::prelude::*;
use leptos::task::spawn_local;
use tracing::error;

use crate::core::updater;
use crate::i18n::{native_language_to_locale, use_i18n};
use crate::pages::login::oauth_listeners::{check_url_oauth_callback, setup_oauth_listener};
use crate::routes::AppRoutes;
use crate::store::auth_store::AuthStore;
use crate::store::connectivity::ConnectivityStore;
use crate::ui_components::{
    ConnectivityBanner, LoadingOverlay, ToastContainer, ToastData, UpdateDrawer,
};

#[component]
pub fn App() -> impl IntoView {
    let auth_store = AuthStore::new();
    let connectivity = ConnectivityStore::new();
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let disposed = StoredValue::new(());

    provide_context(auth_store.repository().clone());
    provide_context(auth_store.clone());
    provide_context(connectivity);

    let i18n = use_i18n();

    auth_store.check_session();
    check_url_oauth_callback(&auth_store, &i18n);
    setup_oauth_listener(auth_store.clone(), i18n);

    let update_info = RwSignal::new(None::<updater::UpdateInfo>);
    let download_progress = RwSignal::new(None::<f32>);

    let update_info_clone = update_info;
    spawn_local(async move {
        if let Some(info) = updater::check_for_updates().await {
            if disposed.is_disposed() {
                return;
            }
            update_info_clone.set(Some(info));
        }
    });

    let i18n_for_lang = i18n;
    Effect::new(move |_| {
        if let Some(user) = auth_store.user.get() {
            let locale = native_language_to_locale(user.native_language());
            i18n_for_lang.set_locale(locale);
        }
    });

    let on_update = Callback::new(move |_| {
        let disposed = StoredValue::new(());
        spawn_local(async move {
            download_progress.set(Some(0.0));

            let result = updater::download_and_install(move |progress| {
                download_progress.set(Some(progress as f32));
            })
            .await;

            if disposed.is_disposed() {
                return;
            }
            if let Err(e) = result {
                error!("Update failed: {}", e);
                download_progress.set(None);
            }
        });
    });

    let auth_store_for_oauth = auth_store.clone();

    view! {
        <ConnectivityBanner />
        {move || update_info.get().map(|info| view! {
            <UpdateDrawer
                current_version=info.current_version
                new_version=info.version
                on_update=on_update
                download_progress=Signal::from(download_progress)
            />
        })}
        <ToastContainer toasts=toasts duration_ms=5000 />
        <Show when=move || auth_store_for_oauth.is_oauth_loading.get()>
            {move || {
                let message = i18n.get_keys().app().logging_in().inner().to_string();
                view! { <LoadingOverlay message=message /> }
            }}
        </Show>
        <AppRoutes />
    }
}

#[allow(dead_code)]
fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}
