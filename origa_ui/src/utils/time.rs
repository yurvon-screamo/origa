/// Returns current time in milliseconds from Performance API.
/// Returns 0.0 if Performance API is unavailable (e.g., SSR).
pub fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}
