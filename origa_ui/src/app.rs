use leptos::prelude::*;
use leptos::task::spawn_local;
use tracing::{error, info};

use crate::core::updater;
use crate::loaders::{load_all_data, load_dictionary};
use crate::pages::login::oauth_listeners::{check_url_oauth_callback, setup_oauth_listener};
use crate::routes::AppRoutes;
use crate::store::auth_store::AuthStore;
use crate::store::connectivity::ConnectivityStore;
use crate::ui_components::{ConnectivityBanner, LoadingOverlay, ToastContainer, ToastData, ToastType, UpdateDrawer};

const DICT_TOAST_ID: usize = 9999;

#[component]
pub fn App() -> impl IntoView {
    let auth_store = AuthStore::new();
    let connectivity = ConnectivityStore::new();
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());

    provide_context(auth_store.repository().clone());
    provide_context(auth_store.clone());
    provide_context(connectivity);

    auth_store.check_session();
    check_url_oauth_callback(&auth_store);
    setup_oauth_listener(auth_store.clone());

    let update_info = RwSignal::new(None::<updater::UpdateInfo>);
    let download_progress = RwSignal::new(None::<f32>);

    let update_info_clone = update_info;
    spawn_local(async move {
        if let Some(info) = updater::check_for_updates().await {
            update_info_clone.set(Some(info));
        }
    });

    let on_update = Callback::new(move |_| {
        spawn_local(async move {
            download_progress.set(Some(0.0));

            let result = updater::download_and_install(move |progress| {
                download_progress.set(Some(progress as f32));
            })
            .await;

            if let Err(e) = result {
                error!("Update failed: {}", e);
                download_progress.set(None);
            }
        });
    });

    let auth_store_for_init = auth_store.clone();
    let toasts_for_init = toasts;
    spawn_local(async move {
        init_dictionary(auth_store_for_init, toasts_for_init).await;
    });

    let auth_store_for_loading = auth_store.clone();
    let auth_store_for_oauth = auth_store.clone();
    let auth_store_for_data = auth_store.clone();

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
        <Show when=move || auth_store_for_loading.is_loading().get()>
            {move || {
                let message = if auth_store_for_oauth.is_oauth_loading.get() {
                    "Вход..."
                } else if !auth_store_for_data.is_data_loaded.get() {
                    "Загрузка словарей..."
                } else {
                    "Проверка авторизации..."
                };
                view! { <LoadingOverlay message=message /> }
            }}
        </Show>
        <AppRoutes />
    }
}

async fn init_dictionary(auth_store: AuthStore, toasts: RwSignal<Vec<ToastData>>) {
    let start = now_ms();
    info!("🚀 Starting application data initialization...");

    // Load basic data first (fast, ~4s)
    let dict_start = now_ms();
    let data_result = load_all_data().await;
    let parallel_end = now_ms();
    info!("⏱️ Basic data loading completed in {:.2}s", (parallel_end - dict_start) / 1000.0);

    if let Err(e) = data_result {
        error!("Failed to load data: {:?}", e);
    } else {
        info!("✅ All basic data loaded ({:.2}s)", (parallel_end - dict_start) / 1000.0);
    }

    // Mark data as loaded so UI can show immediately
    auth_store.set_data_loaded();
    info!("🎉 App ready ({:.2}s)", (now_ms() - start) / 1000.0);

    // Load dictionary in background (slow, ~17s)
    init_background_dictionary(auth_store, toasts);
}

fn init_background_dictionary(auth_store: AuthStore, toasts: RwSignal<Vec<ToastData>>) {
    spawn_local(async move {
        // Show loading toast
        toasts.update(|t| {
            t.push(ToastData {
                id: DICT_TOAST_ID,
                title: "Загрузка словаря".to_string(),
                message: "Загружаем словарь токенизации...".to_string(),
                toast_type: ToastType::Info,
                duration_ms: None,
            });
        });

        let start = now_ms();
        info!("📖 Loading dictionary in background...");

        match load_dictionary().await {
            Ok(()) => {
                let elapsed = (now_ms() - start) / 1000.0;
                info!("✅ Dictionary loaded in background ({:.2}s)", elapsed);
                auth_store.set_dictionary_loaded();

                // Update toast to success
                toasts.update(|t| {
                    t.retain(|toast| toast.id != DICT_TOAST_ID);
                    t.push(ToastData {
                        id: DICT_TOAST_ID,
                        title: "Словарь загружен".to_string(),
                        message: format!("Готово к работе ({:.1}с)", elapsed),
                        toast_type: ToastType::Success,
                        duration_ms: Some(3000),
                    });
                });
            }
            Err(e) => {
                error!("Failed to load dictionary: {}", e);

                // Update toast to error
                toasts.update(|t| {
                    t.retain(|toast| toast.id != DICT_TOAST_ID);
                    t.push(ToastData {
                        id: DICT_TOAST_ID,
                        title: "Ошибка загрузки".to_string(),
                        message: "Не удалось загрузить словарь токенизации".to_string(),
                        toast_type: ToastType::Error,
                        duration_ms: Some(5000),
                    });
                });
            }
        }
    });
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}
