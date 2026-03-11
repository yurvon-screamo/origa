use tauri::{Emitter, Listener, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    eprintln!("[ORIGA] Starting Tauri app...");

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            eprintln!("[ORIGA] Single instance activated");
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
            eprintln!("[ORIGA] Setup started");
            let handle = app.handle().clone();
            let handle_for_event = app.handle().clone();

            // Способ 1: слушаем событие от плагина (как в kanri)
            eprintln!("[ORIGA] Registering deep-link://new-url listener");
            app.listen("deep-link://new-url", move |event: tauri::Event| {
                let payload = event.payload();
                eprintln!("[ORIGA] deep-link://new-url event received: {}", payload);

                if let Ok(urls) = serde_json::from_str::<Vec<String>>(payload) {
                    for url in urls {
                        eprintln!("[ORIGA] Processing URL: {}", url);
                        if url.starts_with("origa://") {
                            eprintln!("[ORIGA] Emitting deep-link-received: {}", url);
                            let _ = handle_for_event.emit("deep-link-received", &url);
                        }
                    }
                } else {
                    eprintln!("[ORIGA] Failed to parse payload as JSON array");
                }
            });

            // Способ 2: on_open_url callback
            let handle_for_callback = handle.clone();
            eprintln!("[ORIGA] Registering on_open_url callback");
            app.deep_link().on_open_url(move |event| {
                eprintln!("[ORIGA] on_open_url callback triggered");
                for url in event.urls() {
                    eprintln!("[ORIGA] URL in on_open_url: {}", url);
                    if url.scheme() == "origa" {
                        eprintln!("[ORIGA] Emitting deep-link-received from on_open_url");
                        let _ = handle_for_callback.emit("deep-link-received", url.to_string());
                    }
                }
            });

            // Обработка deep-link при запуске приложения
            #[cfg(desktop)]
            {
                eprintln!("[ORIGA] Checking for startup deep-link");
                let handle_for_startup = app.handle().clone();
                match app.deep_link().get_current() {
                    Ok(Some(urls)) => {
                        eprintln!("[ORIGA] Startup deep-links found: {:?}", urls);
                        if let Some(url) = urls.first() {
                            eprintln!("[ORIGA] App started via deep link: {}", url);
                            let _ = handle_for_startup.emit("deep-link-received", url.to_string());
                        }
                    }
                    Ok(None) => {
                        eprintln!("[ORIGA] No startup deep-links");
                    }
                    Err(e) => {
                        eprintln!("[ORIGA] Error getting startup deep-links: {:?}", e);
                    }
                }
            }

            #[cfg(any(windows, target_os = "linux"))]
            {
                eprintln!("[ORIGA] Registering deep-link schemes");
                match app.deep_link().register_all() {
                    Ok(_) => eprintln!("[ORIGA] Deep-link schemes registered successfully"),
                    Err(e) => eprintln!("[ORIGA] Failed to register deep-link schemes: {:?}", e),
                }
            }

            eprintln!("[ORIGA] Setup completed");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
