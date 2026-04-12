#[cfg(feature = "stt")]
use base64::Engine;
use tauri::{Emitter, Listener, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(feature = "stt")]
#[tauri::command]
fn transcribe_audio(
    app: tauri::AppHandle,
    audio_base64: String,
    file_name: String,
) -> Result<String, String> {
    let model_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?
        .join("whisper");

    if !model_dir.exists() {
        return Err(format!(
            "Whisper model directory not found: {:?}",
            model_dir
        ));
    }

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&audio_base64)
        .map_err(|e| format!("Failed to decode base64 audio: {}", e))?;

    let extension = file_name.rsplit('.').next_back().unwrap_or("wav");

    let temp_path =
        std::env::temp_dir().join(format!("origa_audio_{}.{}", file_name.len(), extension));

    std::fs::write(&temp_path, &bytes)
        .map_err(|e| format!("Failed to write temp audio file: {}", e))?;

    let transcriber = origa::stt::WhisperTranscriber::new(&model_dir)
        .map_err(|e| format!("Failed to initialize Whisper: {}", e))?;

    let text = transcriber
        .transcribe(&temp_path)
        .map_err(|e| format!("Transcription failed: {}", e))?;

    let _ = std::fs::remove_file(&temp_path);

    Ok(text)
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

    let builder = builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_tts::init());

    #[cfg(feature = "stt")]
    let builder = builder.invoke_handler(tauri::generate_handler![greet, transcribe_audio]);

    #[cfg(not(feature = "stt"))]
    let builder = builder.invoke_handler(tauri::generate_handler![greet]);

    builder
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
