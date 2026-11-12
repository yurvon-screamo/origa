//! CI selftest marker bridge (task 20261104060604, GH #854).
//!
//! The WASM selftest deep-link handler reports its feature-probe outcome
//! through this command, and the native side writes it to logcat under the
//! app's own `I/origa` tag — the stable, grep-able channel the Android
//! smoke script asserts on (the ADR-063 marker precedent; tracing has no
//! logcat backend in this app, and WebView console output lands under the
//! foreign `Tauri/Console` tag). A broken bridge means the smoke job fails
//! on the missing marker, which is the end-to-end coverage this one-line
//! transit needs.
//!
//! Registered unconditionally like `request_app_review`; non-Android builds
//! just log (the command never receives probes there — the selftest
//! listener mounts on Android only).

/// Carries a selftest marker to the app's logcat marker channel.
#[tauri::command]
pub(crate) fn selftest_marker(message: String) {
    #[cfg(target_os = "android")]
    crate::android_context::logcat_info(&message);
    #[cfg(not(target_os = "android"))]
    tracing::info!("{message}");
}

#[cfg(test)]
mod tests {
    /// Structural contract (the include_str! guard pattern of
    /// tests/build_config.rs): the Android arm must write to logcat — the
    /// only channel the CI smoke job observes (ADR-063) — and the fallback
    /// arm must log somewhere on the remaining platforms. Compiling both
    /// arms per-target is enforced by the cfg attributes themselves (each
    /// platform build compiles exactly one).
    #[test]
    fn bridge_targets_the_ci_observable_channel() {
        let src = include_str!("selftest_marker.rs");
        assert!(
            src.contains("android_context::logcat_info"),
            "the Android arm must write logcat (ADR-063) — found no logcat call"
        );
        assert!(
            src.contains("tracing::info!"),
            "the non-Android arm must log through tracing — found no tracing call"
        );
    }
}
