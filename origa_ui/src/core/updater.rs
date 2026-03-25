use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
}

#[cfg(all(
    target_arch = "wasm32",
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
fn get_js_property(
    obj: &leptos::wasm_bindgen::JsValue,
    name: &str,
    error_msg: &str,
) -> Option<leptos::wasm_bindgen::JsValue> {
    let result = js_sys::Reflect::get(obj, &leptos::wasm_bindgen::JsValue::from_str(name));
    if result.is_err() {
        leptos::logging::warn!("{}", error_msg);
    }
    result.ok()
}

pub async fn check_for_updates() -> Option<UpdateInfo> {
    #[cfg(all(
        target_arch = "wasm32",
        any(target_os = "windows", target_os = "macos", target_os = "linux")
    ))]
    {
        check_for_updates_tauri().await
    }

    #[cfg(not(all(
        target_arch = "wasm32",
        any(target_os = "windows", target_os = "macos", target_os = "linux")
    )))]
    {
        leptos::logging::log!("Проверка обновлений доступна только в desktop-версии");
        None
    }
}

#[cfg(all(
    target_arch = "wasm32",
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
async fn check_for_updates_tauri() -> Option<UpdateInfo> {
    use super::version::VERSION;
    use leptos::logging;
    use leptos::wasm_bindgen::JsCast;
    use leptos::wasm_bindgen::JsValue;

    let window = web_sys::window();
    let tauri_obj = window
        .as_ref()
        .and_then(|w| get_js_property(w, "__TAURI__", "Tauri API недоступен"));
    let updater_mod = tauri_obj
        .as_ref()
        .and_then(|obj| get_js_property(obj, "updater", "Tauri updater модуль недоступен"));
    let check_fn_val = updater_mod
        .as_ref()
        .and_then(|mod_| get_js_property(mod_, "check", "Tauri updater.check недоступен"));
    let check_fn = check_fn_val.as_ref().and_then(|val| {
        let fn_result = val.dyn_into::<js_sys::Function>().ok();
        if fn_result.is_none() {
            logging::warn!("updater.check не является функцией");
        }
        fn_result
    });

    let result = check_fn
        .as_ref()
        .map(|fn_val| fn_val.call0(&JsValue::UNDEFINED));

    match result {
        Some(Ok(promise)) => {
            let promise_result =
                wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await;
            match promise_result {
                Ok(update_result) => {
                    if update_result.is_null() || update_result.is_undefined() {
                        logging::log!("Обновления не найдены");
                        None
                    } else {
                        parse_update_info(&update_result)
                    }
                },
                Err(e) => {
                    logging::warn!("Ошибка при проверке обновлений: {:?}", e);
                    None
                },
            }
        },
        Some(Err(e)) => {
            logging::warn!("Ошибка вызова updater.check: {:?}", e);
            None
        },
        None => {
            if window.is_none() {
                logging::warn!("Window недоступен");
            }
            None
        },
    }
}

#[cfg(all(
    target_arch = "wasm32",
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
fn parse_update_info(value: &leptos::wasm_bindgen::JsValue) -> Option<UpdateInfo> {
    use super::version::VERSION;
    use leptos::wasm_bindgen::JsValue;

    let Ok(version) = js_sys::Reflect::get(value, &JsValue::from_str("version")) else {
        return None;
    };

    let version_str = version.as_string()?;

    Some(UpdateInfo {
        version: version_str,
        current_version: VERSION.to_string(),
    })
}

pub async fn download_and_install<F>(_on_progress: F) -> Result<(), String>
where
    F: Fn(u8) + Send + Sync + 'static,
{
    #[cfg(all(
        target_arch = "wasm32",
        any(target_os = "windows", target_os = "macos", target_os = "linux")
    ))]
    {
        download_and_install_tauri(_on_progress).await
    }

    #[cfg(not(all(
        target_arch = "wasm32",
        any(target_os = "windows", target_os = "macos", target_os = "linux")
    )))]
    {
        leptos::logging::log!("Загрузка обновлений доступна только в desktop-версии");
        Ok(())
    }
}

#[cfg(all(
    target_arch = "wasm32",
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
async fn download_and_install_tauri<F>(on_progress: F) -> Result<(), String>
where
    F: Fn(u8) + Send + Sync + 'static,
{
    use leptos::logging;
    use leptos::wasm_bindgen::closure::Closure;
    use leptos::wasm_bindgen::JsCast;
    use leptos::wasm_bindgen::JsValue;
    use std::sync::Arc;

    let window = web_sys::window().ok_or("Window недоступен")?;

    let tauri_obj = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "Tauri API недоступен")?;

    let updater_mod = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("updater"))
        .map_err(|_| "Tauri updater модуль недоступен")?;

    let download_fn = js_sys::Reflect::get(&updater_mod, &JsValue::from_str("downloadAndInstall"))
        .map_err(|_| "Tauri updater.downloadAndInstall недоступен")?;

    let download_fn = download_fn
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "updater.downloadAndInstall не является функцией")?;

    let on_progress = Arc::new(on_progress);
    let closure = Closure::<dyn Fn(JsValue)>::new(move |event| {
        if let Ok(data) = js_sys::Reflect::get(&event, &JsValue::from_str("data")) {
            if let Some(content_length) =
                js_sys::Reflect::get(&data, &JsValue::from_str("contentLength"))
                    .ok()
                    .and_then(|v| v.as_f64())
            {
                if content_length > 0.0 {
                    if let Some(chunk_length) =
                        js_sys::Reflect::get(&data, &JsValue::from_str("chunkLength"))
                            .ok()
                            .and_then(|v| v.as_f64())
                    {
                        let progress = ((chunk_length / content_length) * 100.0).min(100.0) as u8;
                        on_progress(progress);
                    }
                }
            }
        }
    });

    let event_name = JsValue::from_str("tauri://update-download-progress");
    let event_mod = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("event"))
        .map_err(|_| "Tauri event модуль недоступен")?;

    let listen_fn = js_sys::Reflect::get(&event_mod, &JsValue::from_str("listen"))
        .map_err(|_| "Tauri event.listen недоступен")?;

    let listen_fn = listen_fn
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "event.listen не является функцией")?;

    let _unlisten = listen_fn.call2(&JsValue::UNDEFINED, &event_name, closure.as_ref());
    closure.forget();

    let result = download_fn.call0(&JsValue::UNDEFINED);

    match result {
        Ok(promise) => {
            let promise = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise));
            promise
                .await
                .map(|_| ())
                .map_err(|e| format!("Ошибка при загрузке обновления: {:?}", e))
        },
        Err(e) => Err(format!("Ошибка вызова downloadAndInstall: {:?}", e)),
    }
}
