use leptos::prelude::*;
use leptos::task::spawn_local;
use tracing::{error, info};

use crate::core::updater;
use crate::loaders::{load_all_data, load_dictionary};
use crate::pages::login::oauth_listeners::{check_url_oauth_callback, setup_oauth_listener};
use crate::routes::AppRoutes;
use crate::store::auth_store::AuthStore;
use crate::ui_components::LoadingOverlay;
use crate::ui_components::UpdateDrawer;

#[component]
pub fn App() -> impl IntoView {
    let auth_store = AuthStore::new();

    provide_context(auth_store.repository().clone());
    provide_context(auth_store.clone());

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
    spawn_local(async move {
        init_dictionary(auth_store_for_init).await;
    });

    let auth_store_for_loading = auth_store.clone();
    let auth_store_for_oauth = auth_store.clone();
    let auth_store_for_data = auth_store.clone();

    view! {
        {move || update_info.get().map(|info| view! {
            <UpdateDrawer
                current_version=info.current_version
                new_version=info.version
                on_update=on_update
                download_progress=Signal::from(download_progress)
            />
        })}
        <Show when=move || auth_store_for_loading.is_loading().get()>
            <LoadingOverlay message="Проверка авторизации..." />
        </Show>
        <Show when=move || auth_store_for_oauth.is_oauth_loading.get()>
            <LoadingOverlay message="Вход..." />
        </Show>
        <Show when=move || !auth_store_for_data.is_data_loaded.get()>
            <LoadingOverlay message="Загрузка словарей..." />
        </Show>
        <AppRoutes />
    }
}

async fn init_dictionary(auth_store: AuthStore) {
    let (dict_result, data_result) = futures::join!(load_dictionary(), load_all_data());

    if let Err(e) = dict_result {
        error!("Failed to load dictionary: {}", e);
    } else {
        info!("Unidic dictionary loaded");
    }
    if let Err(e) = data_result {
        error!("Failed to load data: {:?}", e);
    } else {
        info!("All data loaded");
    }

    auth_store.set_data_loaded();
}
