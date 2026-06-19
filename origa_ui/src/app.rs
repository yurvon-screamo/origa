use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};

use crate::core::updater;
use crate::i18n::{native_language_to_locale, use_i18n};
use crate::pages::login::oauth_listeners::{check_url_oauth_callback, setup_oauth_listener};
use crate::repository::migrate_session_to_store_if_needed;
use crate::routes::AppRoutes;
use crate::store::auth_store::AuthStore;
use crate::store::connectivity::ConnectivityStore;
use crate::store::offline_bundle_store::OfflineBundleStore;
use crate::ui_components::{
    ConnectivityBanner, LoadingOverlay, ToastContainer, ToastData, UpdateDrawer,
};

#[component]
pub fn App() -> impl IntoView {
    let auth_store = AuthStore::new();
    let connectivity = ConnectivityStore::new();
    let offline_bundle_store = OfflineBundleStore::new();
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let disposed = StoredValue::new(());

    provide_context(auth_store.repository().clone());
    provide_context(auth_store.clone());
    provide_context(connectivity);
    provide_context(offline_bundle_store);

    let i18n = use_i18n();
    let navigate = use_navigate();

    // AD-1 + AD-4: migrate localStorage → Tauri store (one-time), then check
    // session. Migration MUST complete before check_session reads the store so
    // existing users upgrading from pre-ADR-010 builds keep their session.
    let auth_store_for_session = auth_store.clone();
    spawn_local(async move {
        debug!("session lifecycle: starting migration + check_session");
        migrate_session_to_store_if_needed().await;
        auth_store_for_session.check_session();
    });

    // AD-4: OAuth callback checks run concurrently with session check.
    // check_url_oauth_callback handles the web-build URL-fragment path;
    // setup_oauth_listener handles the Tauri deep-link path (both cold-start
    // pending links and live events).
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

    // AD-3: SPA navigate Effect — always mounted in App() (unlike the Login
    // page which can unmount). Watches is_authenticated() and navigates to
    // /home when the user becomes authenticated. This covers BOTH the
    // email/password path (where user.set is called explicitly in login())
    // and the OAuth path (where set_oauth_session writes the session
    // asynchronously via IPC + Store::save()). Replaces the old
    // window.location.set_href("/home") which caused a full WebView reload
    // with loss of reactive state.
    let auth_store_for_nav = auth_store.clone();
    Effect::new(move |_| {
        if auth_store_for_nav.is_authenticated().get() {
            debug!("redirect_decision: authenticated → navigate to /home");
            navigate("/home", Default::default());
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
