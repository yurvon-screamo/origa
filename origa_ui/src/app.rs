use leptos::prelude::*;
use leptos::task::spawn_local;
use tracing::{error, info};

use crate::core::updater;
use crate::loaders::data_loader::{load_grammar, load_kanji, load_radical, load_vocabulary};
use crate::loaders::jlpt_content_loader::load_jlpt_content;
use crate::loaders::load_dictionary;
use crate::pages::login::oauth_listeners::{check_url_oauth_callback, setup_oauth_listener};
use crate::routes::AppRoutes;
use crate::store::auth_store::AuthStore;
use crate::store::connectivity::ConnectivityStore;
use crate::ui_components::{
    AppSkeleton, ConnectivityBanner, LoadingOverlay, ToastContainer, ToastData, ToastType,
    UpdateDrawer,
};
use crate::utils::yield_to_browser;

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

    let auth_store_for_oauth = auth_store.clone();
    let auth_store_for_checking = auth_store.clone();
    let auth_store_for_dictionary = auth_store.clone();

    view! {
        <ConnectivityBanner />
        // AppSkeleton блокирует UI пока словарь не загружен
        <Show when=move || !auth_store_for_dictionary.is_dictionary_loaded.get()>
            <AppSkeleton />
        </Show>

        // OAuth и check_session overlay
        <Show when=move || auth_store_for_oauth.is_oauth_loading.get() || auth_store_for_checking.is_checking_session.get()>
            {move || {
                let message = if auth_store_for_oauth.is_oauth_loading.get() {
                    "Вход..."
                } else {
                    "Проверка авторизации..."
                };
                view! { <LoadingOverlay message=message /> }
            }}
        </Show>

        // Остальной UI
        {move || update_info.get().map(|info| view! {
            <UpdateDrawer
                current_version=info.current_version
                new_version=info.version
                on_update=on_update
                download_progress=Signal::from(download_progress)
            />
        })}
        <ToastContainer toasts=toasts duration_ms=5000 />
        <AppRoutes />
    }
}

async fn init_dictionary(auth_store: AuthStore, toasts: RwSignal<Vec<ToastData>>) {
    let start = now_ms();
    info!("🚀 Starting application data initialization...");

    const PROGRESS_TOAST_ID: usize = 9998;
    let total = 6;
    let mut loaded_count = 0;

    let update_progress_toast = |count: usize, msg: &str| {
        toasts.update(|t| {
            t.retain(|toast| toast.id != PROGRESS_TOAST_ID);
            t.push(ToastData {
                id: PROGRESS_TOAST_ID,
                toast_type: ToastType::Info,
                title: "Загрузка".to_string(),
                message: format!("{} ({}/{})", msg, count, total),
                duration_ms: None,
                closable: false,
            });
        });
    };

    update_progress_toast(1, "Загрузка словаря");
    yield_to_browser().await;

    match load_vocabulary().await {
        Ok(()) => {
            loaded_count += 1;
            info!("✅ Vocabulary loaded");
            update_progress_toast(loaded_count + 1, "Загрузка канджи");
        }
        Err(e) => {
            error!("Failed to load vocabulary: {:?}", e);
        }
    }
    yield_to_browser().await;

    match load_kanji().await {
        Ok(()) => {
            loaded_count += 1;
            info!("✅ Kanji loaded");
            update_progress_toast(loaded_count + 1, "Загрузка радикалов");
        }
        Err(e) => {
            error!("Failed to load kanji: {:?}", e);
        }
    }
    yield_to_browser().await;

    match load_radical().await {
        Ok(()) => {
            loaded_count += 1;
            info!("✅ Radicals loaded");
            update_progress_toast(loaded_count + 1, "Загрузка грамматики");
        }
        Err(e) => {
            error!("Failed to load radicals: {:?}", e);
        }
    }
    yield_to_browser().await;

    match load_grammar().await {
        Ok(()) => {
            loaded_count += 1;
            info!("✅ Grammar loaded");
            update_progress_toast(loaded_count + 1, "Загрузка JLPT");
        }
        Err(e) => {
            error!("Failed to load grammar: {:?}", e);
        }
    }
    yield_to_browser().await;

    match load_jlpt_content().await {
        Ok(()) => {
            loaded_count += 1;
            info!("✅ JLPT content loaded");
            update_progress_toast(loaded_count + 1, "Загрузка словаря токенизации");
        }
        Err(e) => {
            error!("Failed to load JLPT content: {:?}", e);
        }
    }
    yield_to_browser().await;

    let has_error = loaded_count < 4;

    if has_error {
        show_error_toast(&toasts, "Не удалось загрузить критические данные");
        return;
    }

    auth_store.set_data_loaded();
    info!("✅ Basic data loaded ({:.2}s)", (now_ms() - start) / 1000.0);

    match load_dictionary().await {
        Ok(()) => {
            let elapsed = (now_ms() - start) / 1000.0;
            auth_store.set_dictionary_loaded();
            info!("✅ Dictionary loaded ({:.2}s total)", elapsed);
            show_success_toast(&toasts, &format!("Данные загружены ({:.1}с)", elapsed));
        }
        Err(e) => {
            error!("Failed to load dictionary: {}", e);
            show_error_toast(&toasts, "Не удалось загрузить словарь токенизации");
        }
    }

    info!("🎉 App ready ({:.2}s)", (now_ms() - start) / 1000.0);
}

fn show_success_toast(toasts: &RwSignal<Vec<ToastData>>, message: &str) {
    toasts.update(|t| {
        t.retain(|toast| toast.id != DICT_TOAST_ID);
        t.push(ToastData {
            id: DICT_TOAST_ID,
            toast_type: ToastType::Success,
            title: "Готово".to_string(),
            message: message.to_string(),
            duration_ms: Some(3000),
            closable: true,
        });
    });
}

fn show_error_toast(toasts: &RwSignal<Vec<ToastData>>, message: &str) {
    toasts.update(|t| {
        t.retain(|toast| toast.id != DICT_TOAST_ID);
        t.push(ToastData {
            id: DICT_TOAST_ID,
            toast_type: ToastType::Error,
            title: "Ошибка".to_string(),
            message: message.to_string(),
            duration_ms: Some(5000),
            closable: true,
        });
    });
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}
