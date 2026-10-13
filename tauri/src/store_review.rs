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

/// Requests the native store review prompt.
///
/// Returns `true` when the platform supports in-app review and the prompt
/// was handed to the OS (the OS decides whether to actually show it —
/// Apple caps prompts at 3 per 365 days and silently drops excess calls).
/// Returns `false` on platforms without a wired review API, so the
/// frontend keeps its cooldown intact for future sessions.
#[tauri::command]
pub fn request_app_review(app: tauri::AppHandle) -> bool {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        return request_app_review_apple(app);
    }
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        let _ = app;
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
            eprintln!("[store-review] UIApplication unavailable");
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
