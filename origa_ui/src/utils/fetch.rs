use leptos::wasm_bindgen::JsCast;
use origa::domain::OrigaError;
use wasm_bindgen_futures::JsFuture;

pub async fn fetch_text(url: impl Into<String>) -> Result<String, OrigaError> {
    let url = url.into();

    let window = web_sys::window().ok_or_else(|| OrigaError::NetworkError {
        url: url.clone(),
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("Failed to fetch: {:?}", e),
        })?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|e| OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("Failed to cast response: {:?}", e),
        })?;

    if !resp.ok() {
        return Err(OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("HTTP {}", resp.status()),
        });
    }

    let text = JsFuture::from(resp.text().map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to get text promise: {:?}", e),
    })?)
    .await
    .map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to read text: {:?}", e),
    })?;

    text.as_string().ok_or_else(|| OrigaError::NetworkError {
        url: url.clone(),
        reason: "Response is not a string".to_string(),
    })
}
