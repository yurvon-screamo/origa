mod auth_store;
mod updater_commands;

use std::sync::Mutex;

use auth_store::{auth_store_delete, auth_store_get, auth_store_set};
use tauri::{Emitter, Listener, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use updater_commands::{PendingUpdate, check_for_update, install_update};

struct PendingDeepLink(Mutex<Option<String>>);

#[tauri::command]
fn get_pending_deep_link(state: tauri::State<'_, PendingDeepLink>) -> Option<String> {
    state.0.lock().unwrap_or_else(|e| e.into_inner()).take()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tracing::info!("[deep-link] single-instance activated (app was already running)");
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
        builder = builder.plugin(tauri_plugin_process::init());
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_tts::init())
        .manage(PendingDeepLink(Mutex::new(None)))
        .manage(PendingUpdate::new())
        .invoke_handler(tauri::generate_handler![
            get_pending_deep_link,
            auth_store_get,
            auth_store_set,
            auth_store_delete,
            #[cfg(desktop)]
            check_for_update,
            #[cfg(desktop)]
            install_update
        ])
        .setup(|app| {
            tracing::info!("[deep-link] setup started");

            let handle_for_event = app.handle().clone();

            app.listen("deep-link://new-url", move |event: tauri::Event| {
                let payload = event.payload();
                tracing::info!(
                    "[deep-link] received 'deep-link://new-url' event, payload: {}",
                    payload
                );

                if let Ok(urls) = serde_json::from_str::<Vec<String>>(payload) {
                    tracing::info!("[deep-link] parsed {} url(s) from payload", urls.len());
                    for url in &urls {
                        tracing::info!("[deep-link] checking url: {}", url);
                        if url.starts_with("origa://") {
                            tracing::info!(
                                "[deep-link] emitting 'deep-link-received' with url: {}",
                                url
                            );
                            let _ = handle_for_event.emit("deep-link-received", url);
                        }
                    }
                } else {
                    tracing::error!(
                        "[deep-link] failed to parse payload as Vec<String>: {}",
                        payload
                    );
                }
            });

            tracing::info!("[deep-link] listener for 'deep-link://new-url' registered");

            {
                let pending = app.state::<PendingDeepLink>();
                match app.deep_link().get_current() {
                    Ok(Some(urls)) => {
                        tracing::info!(
                            "[deep-link] get_current() returned {} url(s) at startup",
                            urls.len()
                        );
                        if let Some(url) = urls.first() {
                            tracing::info!(
                                "[deep-link] saved startup deep-link to pending: {}",
                                url
                            );
                            *pending.0.lock().unwrap_or_else(|e| e.into_inner()) =
                                Some(url.to_string());
                        }
                    },
                    Ok(None) => {
                        tracing::info!("[deep-link] get_current() returned no urls at startup");
                    },
                    Err(e) => {
                        tracing::warn!("[deep-link] get_current() error at startup: {:?}", e);
                    },
                }
            }

            #[cfg(any(windows, target_os = "linux"))]
            {
                match app.deep_link().register_all() {
                    Ok(()) => {
                        tracing::info!(
                            "[deep-link] register_all() succeeded — scheme 'origa://' is registered"
                        );
                    },
                    Err(e) => {
                        tracing::error!(
                            "[deep-link] register_all() FAILED: {:?} — deep links will NOT work!",
                            e
                        );
                    },
                }
            }

            tracing::info!("[deep-link] setup complete");

            #[cfg(feature = "release-devtools")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                    tracing::info!("[devtools] DevTools opened (release-devtools feature enabled)");
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
