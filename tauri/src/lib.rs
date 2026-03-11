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
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let handle = app.handle().clone();
            let handle_for_event = app.handle().clone();

            app.listen("deep-link://new-url", move |event: tauri::Event| {
                let payload = event.payload();
                if let Ok(urls) = serde_json::from_str::<Vec<String>>(payload) {
                    for url in urls {
                        if url.starts_with("origa://") {
                            let _ = handle_for_event.emit("deep-link-received", &url);
                        }
                    }
                }
            });

            let handle_for_callback = handle.clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    if url.scheme() == "origa" {
                        let _ = handle_for_callback.emit("deep-link-received", url.to_string());
                    }
                }
            });

            #[cfg(desktop)]
            {
                let handle_for_startup = app.handle().clone();
                if let Ok(Some(urls)) = app.deep_link().get_current() {
                    if let Some(url) = urls.first() {
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
