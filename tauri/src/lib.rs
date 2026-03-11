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
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let handle = app.handle().clone();
            let handle_for_event = app.handle().clone();

            // Способ 1: слушаем событие от плагина (как в kanri)
            app.listen("deep-link://new-url", move |event: tauri::Event| {
                let payload = event.payload();
                log::info!("deep-link://new-url event received: {}", payload);

                if let Ok(urls) = serde_json::from_str::<Vec<String>>(payload) {
                    for url in urls {
                        if url.starts_with("origa://") {
                            log::info!("Emitting deep-link-received: {}", url);
                            let _ = handle_for_event.emit("deep-link-received", url);
                        }
                    }
                }
            });

            // Способ 2: on_open_url callback
            let handle_for_callback = handle.clone();
            app.deep_link().on_open_url(move |event| {
                log::info!("on_open_url callback triggered");
                for url in event.urls() {
                    log::info!("URL in on_open_url: {}", url);
                    if url.scheme() == "origa" {
                        let _ = handle_for_callback.emit("deep-link-received", url.to_string());
                    }
                }
            });

            // Обработка deep-link при запуске приложения
            #[cfg(desktop)]
            {
                let handle_for_startup = app.handle().clone();
                if let Ok(Some(urls)) = app.deep_link().get_current() {
                    if let Some(url) = urls.first() {
                        log::info!("App started via deep link: {}", url);
                        let _ = handle_for_startup.emit("deep-link-received", url.to_string());
                    }
                }
            }

            #[cfg(any(windows, target_os = "linux"))]
            {
                app.deep_link().register_all()?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
