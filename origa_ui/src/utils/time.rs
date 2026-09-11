/// Milliseconds since an arbitrary monotonic-enough epoch. On WASM this is
/// `performance.now()`; on native targets `web_sys::window()` cannot be
/// touched (imported JS statics panic), so the Unix epoch is used instead.
/// Only differences matter to callers.
pub fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0)
    }
}
