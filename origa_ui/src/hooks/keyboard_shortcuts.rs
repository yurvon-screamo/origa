//! Global keyboard shortcut interception for the Tauri WebView.
//!
//! On Android, unhandled browser accelerators (Ctrl+R, Cmd+R, F5) can trigger a
//! top-level reload that leaks the WebView origin (`tauri.localhost`) to the
//! system browser. With a hardware keyboard attached — the precondition for
//! this bug — Android WebView delivers key events to the page's DOM, so a
//! `keydown` listener can intercept them. This module does so at the earliest
//! JS interception point and redirects the reload to a native WebView reload.

use leptos::prelude::*;
use leptos_use::use_event_listener;

/// Returns `true` if the keyboard event is a reload accelerator.
///
/// Matches: Ctrl+R, Cmd+R, F5 (and their Shift variants — hard-reload is still
/// a reload and must be intercepted, so Shift is intentionally ignored).
/// Does NOT match: plain R, Ctrl+C, Ctrl+S, Enter, etc.
pub(crate) fn is_reload_accelerator(code: &str, ctrl: bool, meta: bool) -> bool {
    let is_r = code == "KeyR";
    let is_f5 = code == "F5";
    (is_r && (ctrl || meta)) || is_f5
}

/// Install a global `keydown` listener that intercepts reload accelerators.
///
/// Inside a Tauri WebView the handler calls `prevent_default()` +
/// `stop_propagation()` (cancelling the accelerator's default reload handling)
/// and then triggers a reload via [`crate::core::tauri::reload_current_webview`]
/// so it stays inside the WebView.
///
/// Outside a Tauri WebView (web dev via `trunk serve`) this is a no-op, leaving
/// the browser's native reload intact for the developer workflow.
pub(crate) fn install_reload_guard() {
    if !crate::core::tauri::is_tauri() {
        return;
    }

    // use_event_listener (leptos-use 0.18) auto-registers cleanup on the
    // current reactive owner, so the return value can be discarded. App() owns
    // the guard for the whole app lifetime.
    let _ = use_event_listener(window(), leptos::ev::keydown, move |ev| {
        if !is_reload_accelerator(&ev.code(), ev.ctrl_key(), ev.meta_key()) {
            return;
        }

        ev.prevent_default();
        ev.stop_propagation();

        if let Err(e) = crate::core::tauri::reload_current_webview() {
            tracing::error!("reload guard: native reload failed: {e}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_r_is_reload() {
        assert!(is_reload_accelerator("KeyR", true, false));
    }

    #[test]
    fn cmd_r_is_reload() {
        assert!(is_reload_accelerator("KeyR", false, true));
    }

    #[test]
    fn f5_is_reload() {
        assert!(is_reload_accelerator("F5", false, false));
    }

    #[test]
    fn plain_r_is_not_reload() {
        assert!(!is_reload_accelerator("KeyR", false, false));
    }

    #[test]
    fn ctrl_c_is_not_reload() {
        assert!(!is_reload_accelerator("KeyC", true, false));
    }

    #[test]
    fn ctrl_s_is_not_reload() {
        assert!(!is_reload_accelerator("KeyS", true, false));
    }

    #[test]
    fn enter_is_not_reload() {
        assert!(!is_reload_accelerator("Enter", false, false));
    }
}
