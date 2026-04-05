use tauri::{Emitter, Listener, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("[deep-link] single-instance activated (app was already running)");
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            log::info!("[deep-link] setup started");

            let handle_for_event = app.handle().clone();

            app.listen("deep-link://new-url", move |event: tauri::Event| {
                let payload = event.payload();
                log::info!(
                    "[deep-link] received 'deep-link://new-url' event, payload: {}",
                    payload
                );

                if let Ok(urls) = serde_json::from_str::<Vec<String>>(payload) {
                    log::info!("[deep-link] parsed {} url(s) from payload", urls.len());
                    for url in &urls {
                        log::info!("[deep-link] checking url: {}", url);
                        if url.starts_with("origa://") {
                            log::info!(
                                "[deep-link] emitting 'deep-link-received' with url: {}",
                                url
                            );
                            let _ = handle_for_event.emit("deep-link-received", url);
                        }
                    }
                } else {
                    log::error!(
                        "[deep-link] failed to parse payload as Vec<String>: {}",
                        payload
                    );
                }
            });

            log::info!("[deep-link] listener for 'deep-link://new-url' registered");

            {
                let handle_for_startup = app.handle().clone();
                match app.deep_link().get_current() {
                    Ok(Some(urls)) => {
                        log::info!(
                            "[deep-link] get_current() returned {} url(s) at startup",
                            urls.len()
                        );
                        if let Some(url) = urls.first() {
                            log::info!("[deep-link] emitting startup deep-link-received: {}", url);
                            let _ = handle_for_startup.emit("deep-link-received", url.to_string());
                        }
                    },
                    Ok(None) => {
                        log::info!("[deep-link] get_current() returned no urls at startup");
                    },
                    Err(e) => {
                        log::warn!("[deep-link] get_current() error at startup: {:?}", e);
                    },
                }
            }

            #[cfg(any(windows, target_os = "linux"))]
            {
                match app.deep_link().register_all() {
                    Ok(()) => {
                        log::info!(
                            "[deep-link] register_all() succeeded — scheme 'origa://' is registered"
                        );
                    },
                    Err(e) => {
                        log::error!(
                            "[deep-link] register_all() FAILED: {:?} — deep links will NOT work!",
                            e
                        );
                    },
                }
            }

            log::info!("[deep-link] setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
