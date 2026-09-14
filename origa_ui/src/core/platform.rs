//! Runtime platform name for diagnostics (feedback reports, Sentry tags).
//!
//! One of: `windows` | `macos` | `linux` | `android` | `ios` | `web`.
//! Detection combines the Tauri bridge with `navigator` hints — the WebView
//! user agent is unreliable on mobile (desktop-site modes), so the explicit
//! Tauri/mobile checks run first.

/// Stable, lowercase platform identifier.
pub fn platform_name() -> String {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return "unknown".to_string(),
    };

    // Tauri WebView: platform comes from the userAgent the shell reports,
    // disambiguated by the mobile checks below.
    let ua = window
        .navigator()
        .user_agent()
        .unwrap_or_default()
        .to_lowercase();

    let is_tauri = crate::core::tauri::is_tauri();

    if ua.contains("android") {
        return "android".to_string();
    }
    if crate::core::tauri::is_ios() {
        return "ios".to_string();
    }
    if !is_tauri {
        return "web".to_string();
    }
    if ua.contains("windows") {
        return "windows".to_string();
    }
    if ua.contains("mac os") || ua.contains("macintosh") {
        return "macos".to_string();
    }
    if ua.contains("linux") {
        return "linux".to_string();
    }
    "unknown".to_string()
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;

    /// The function must never panic in a browser (no-DOM defensive paths):
    /// wasm-bindgen-test runs in a real browser, so `window()` exists; the
    /// assertion guards the non-empty contract of the returned identifier.
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn platform_name_returns_identifier() {
        let name = platform_name();
        assert!(!name.is_empty());
    }
}
