//! In-app store review request (issue #729, ADR-062).
//!
//! The WASM side invokes [`request_app_review`] after a cooldown-gated
//! successful study session; the native side forwards to the platform's
//! in-app review API and reports whether a prompt was actually requested,
//! so the frontend only burns its cooldown on platforms that support it.
//!
//! Platform scope (ADR-062): macOS/iOS route to `SKStoreReviewController`.
//! The class is deprecated since macOS 15 / iOS 18 and Apple's replacement
//! (`AppStore.requestReview(in:)`, StoreKit 2) is Swift-only — no ObjC
//! call site can avoid the deprecation, so both branches are a conscious
//! compromise (see ADR-062 for the rejected Swift-bridge alternative).
//! Windows/Linux return `false`: no in-app store review API is wired yet.

// get_webview_window is a `Manager` trait method; only the macOS branch
// needs it, so the import is target-scoped to stay warning-free on iOS.
#[cfg(target_os = "macos")]
use tauri::Manager;

use crate::platform_routing::HostPlatform;

/// Pure platform contract: in-app store review is wired only on Apple
/// platforms (`SKStoreReviewController`, ADR-062). The WASM gate keys its
/// cooldown off the `false` return — pinning the matrix here keeps the
/// cooldown contract visible to a host test table instead of being implicit
/// in cfg blocks (task 20261104060604, GH #854).
pub fn supports_in_app_review(platform: HostPlatform) -> bool {
    matches!(platform, HostPlatform::MacOS | HostPlatform::Ios)
}

/// Requests the native store review prompt.
///
/// Returns `true` when the platform supports in-app review and the prompt
/// was handed to the OS (the OS decides whether to actually show it —
/// Apple caps prompts at 3 per 365 days and silently drops excess calls).
/// Returns `false` on platforms without a wired review API, so the
/// frontend keeps its cooldown intact for future sessions.
#[tauri::command]
pub fn request_app_review(app: tauri::AppHandle) -> bool {
    if !supports_in_app_review(HostPlatform::current()) {
        let _ = app;
        return false;
    }
    // Only Apple targets pass the routing check above; the cfg keeps the
    // objc2 call sites (and the `Manager` import) out of non-Apple builds,
    // where the early return has already answered `false` and this tail is
    // unreachable-but-total.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        request_app_review_apple(app)
    }
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        false
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn request_app_review_apple(app: tauri::AppHandle) -> bool {
    // SKStoreReviewController must be used from the main thread
    // (AppKit/UIKit requirement).
    #[cfg(target_os = "macos")]
    let ns_window: Option<usize> = app
        .get_webview_window("main")
        .and_then(|window| window.ns_window().ok())
        .map(|ptr| ptr as usize);

    let dispatched = app.run_on_main_thread(move || {
        #[cfg(target_os = "macos")]
        request_review_macos(ns_window);
        #[cfg(target_os = "ios")]
        request_review_ios();
    });
    if let Err(e) = dispatched {
        tracing::warn!("[store-review] main-thread dispatch failed: {e}");
        return false;
    }
    true
}

/// macOS: `+requestReviewInWindow:` with the main window (available well
/// before the project's macOS 14.0 minimum; deprecated macOS 15 — ADR-062).
#[cfg(target_os = "macos")]
fn request_review_macos(ns_window: Option<usize>) {
    use objc2::runtime::AnyObject;

    let Some(ptr) = ns_window else {
        tracing::warn!("[store-review] main NSWindow unavailable");
        return;
    };
    // The pointer is valid while the main window lives; window teardown
    // ends this app's process, so the closure below cannot observe a
    // dangling window.
    let window = unsafe { &*(ptr as *mut AnyObject) };
    unsafe {
        let _: () = objc2::msg_send![
            objc2::class!(SKStoreReviewController),
            requestReviewInWindow: window
        ];
    }
}

/// iOS: `+requestReviewInScene:` with the foreground-active scene
/// (available well before the project's iOS 26.0 minimum; deprecated
/// iOS 18 — ADR-062).
///
/// Tauri iOS runs a single-window single-scene app, so the first scene in
/// `connectedScenes` is the app's only scene; anything but a foreground-
/// active `UIWindowScene` aborts the request (cooldown stays intact).
#[cfg(target_os = "ios")]
fn request_review_ios() {
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;

    unsafe {
        let application: Option<Retained<AnyObject>> =
            objc2::msg_send![objc2::class!(UIApplication), sharedApplication];
        let Some(application) = application else {
            tracing::warn!("[store-review] UIApplication unavailable");
            return;
        };
        let scenes: Option<Retained<AnyObject>> = objc2::msg_send![&application, connectedScenes];
        let Some(scenes) = scenes else {
            return;
        };
        let scene: Option<Retained<AnyObject>> = objc2::msg_send![&scenes, anyObject];
        let Some(scene) = scene else {
            return;
        };
        let is_window_scene: bool =
            objc2::msg_send![&scene, isKindOfClass: objc2::class!(UIWindowScene)];
        // UISceneActivationStateForegroundActive == 0.
        let activation_state: isize = objc2::msg_send![&scene, activationState];
        if !is_window_scene || activation_state != 0 {
            tracing::warn!(
                "[store-review] no foreground-active UIWindowScene (state: {activation_state})"
            );
            return;
        }
        let _: () = objc2::msg_send![
            objc2::class!(SKStoreReviewController),
            requestReviewInScene: &*scene
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// Host-side contract table (task 20261104060604, GH #854): review is
    /// wired on Apple platforms only. The WASM cooldown gate
    /// (`origa_ui/src/core/review_request.rs`) records the prompt timestamp
    /// only on `true` — flipping any row here silently changes how often
    /// real users get prompted, so it must be an explicit, tested decision.
    #[rstest]
    #[case::macos(HostPlatform::MacOS, true)]
    #[case::ios(HostPlatform::Ios, true)]
    #[case::windows(HostPlatform::Windows, false)]
    #[case::linux(HostPlatform::Linux, false)]
    #[case::android(HostPlatform::Android, false)]
    fn supports_in_app_review_routes_every_platform(
        #[case] platform: HostPlatform,
        #[case] expected: bool,
    ) {
        assert_eq!(supports_in_app_review(platform), expected);
    }

    /// The compiled dispatch agrees with the pure table on the target this
    /// test binary runs on: on non-Apple builds the command must answer
    /// `false` without touching any native API.
    #[test]
    fn routing_check_matches_target_support() {
        // On Apple targets the pure check is `true` and the cfg-gated Apple
        // dispatcher compiles in; elsewhere both are `false`. Either way the
        // pure table and the cfg blocks cannot disagree.
        let platform = HostPlatform::current();
        let apple_target = cfg!(any(target_os = "macos", target_os = "ios"));
        assert_eq!(
            supports_in_app_review(platform),
            apple_target,
            "the pure routing table must agree with the compiled target"
        );
    }
}
