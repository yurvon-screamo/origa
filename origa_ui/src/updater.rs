use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
}

pub async fn check_for_updates() -> Option<UpdateInfo> {
    #[cfg(all(target_arch = "wasm32", target_os = "desktop"))]
    {
        check_for_updates_tauri().await
    }

    #[cfg(not(all(target_arch = "wasm32", target_os = "desktop")))]
    {
        leptos::logging::log!("Проверка обновлений доступна только в desktop-версии");
        None
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "desktop"))]
async fn check_for_updates_tauri() -> Option<UpdateInfo> {
    use crate::version::VERSION;
    use leptos::logging;
    use leptos::wasm_bindgen::JsCast;
    use leptos::wasm_bindgen::JsValue;

    let Some(window) = web_sys::window() else {
        logging::warn!("Window недоступен");
        return None;
    };

    let Ok(tauri_obj) = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")) else {
        logging::warn!("Tauri API недоступен");
        return None;
    };

    let Ok(updater_mod) = js_sys::Reflect::get(&tauri_obj, &JsValue::from_str("updater")) else {
        logging::warn!("Tauri updater модуль недоступен");
        return None;
    };

    let Ok(check_fn) = js_sys::Reflect::get(&updater_mod, &JsValue::from_str("check")) else {
        logging::warn!("Tauri updater.check недоступен");
        return None;
    };

    let Ok(check_fn) = check_fn.dyn_into::<js_sys::Function>() else {
        logging::warn!("updater.check не является функцией");
        return None;
    };

    let result = check_fn.call0(&JsValue::UNDEFINED);

    match result {
        Ok(promise) => {
            let promise = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise));
            match promise.await {
                Ok(update_result) => {
                    if update_result.is_null() || update_result.is_undefined() {
                        logging::log!("Обновления не найдены");
                        return None;
                    }

                    parse_update_info(&update_result)
                }
                Err(e) => {
                    logging::warn!("Ошибка при проверке обновлений: {:?}", e);
                    None
                }
            }
        }
        Err(e) => {
            logging::warn!("Ошибка вызова updater.check: {:?}", e);
            None
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "desktop"))]
fn parse_update_info(value: &leptos::wasm_bindgen::JsValue) -> Option<UpdateInfo> {
    use crate::version::VERSION;
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

pub type ProgressCallback = Box<dyn Fn(u8) + Send + Sync>;

pub async fn download_and_install<F>(_on_progress: F) -> Result<(), String>
where
    F: Fn(u8) + Send + Sync + 'static,
{
    #[cfg(all(target_arch = "wasm32", target_os = "desktop"))]
    {
        download_and_install_tauri(_on_progress).await
    }

    #[cfg(not(all(target_arch = "wasm32", target_os = "desktop")))]
    {
        leptos::logging::log!("Загрузка обновлений доступна только в desktop-версии");
        Ok(())
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "desktop"))]
async fn download_and_install_tauri<F>(on_progress: F) -> Result<(), String>
where
    F: Fn(u8) + Send + Sync + 'static,
{
    use leptos::logging;
    use leptos::wasm_bindgen::JsCast;
    use leptos::wasm_bindgen::JsValue;
    use leptos::wasm_bindgen::closure::Closure;
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
            if let Some(content_length) = js_sys::Reflect::get(&data, &JsValue::from_str("contentLength"))
                .ok()
                .and_then(|v| v.as_f64())
            {
                if content_length > 0.0 {
                    if let Some(chunk_length) = js_sys::Reflect::get(&data, &JsValue::from_str("chunkLength"))
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
        }
        Err(e) => Err(format!("Ошибка вызова downloadAndInstall: {:?}", e)),
    }
}
