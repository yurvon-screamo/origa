// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT
//
// Apple-platforms Tauri auth plugin.
//
// 1. `start_auth` wraps ASWebAuthenticationSession for browser-based OAuth
//    flows (Google/Yandex). Unlike plain `open` (which leaves the user
//    stranded in the default browser after the OAuth redirect), it:
//
//    1. Opens the auth page in a dedicated authentication context.
//    2. Intercepts the custom-scheme callback URL directly.
//    3. Returns the callback URL via a completion handler.
//    4. Closes the browser automatically.
//
//    This is exactly what App Review Guideline 4 requires when sign-in
//    leaves the app (Mac App Store rejection, Aug 2026).
//
// 2. `sign_in_with_apple` presents the native Sign in with Apple sheet via
//    ASAuthorizationController (`siwa.rs` on macOS, Swift on iOS): zero web
//    involved, which is what Guideline 4 demands for the Apple button
//    ("Authentication with Sign in with Apple should always be completed
//    without leaving the app").
//
// Platform wiring:
// - iOS: Swift mobile plugin (`ios/Sources`), registered via
//   `register_ios_plugin`; both commands delegate through
//   `run_mobile_plugin`.
// - macOS: pure-Rust implementations in `src/macos.rs` and `src/siwa.rs`
//   via objc2 (`objc2-authentication-services`), dispatched onto the main
//   thread.
// - Other targets: the plugin compiles but both commands return an error —
//   unreachable because the frontend only invokes them on Apple platforms.

use tauri::Runtime;
use tauri::plugin::{Builder, TauriPlugin};

#[cfg(target_os = "ios")]
use tauri::{Manager, plugin::PluginHandle};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_aswebauth);

mod commands;
#[cfg(target_os = "macos")]
pub(crate) mod macos;
#[cfg(target_os = "macos")]
pub(crate) mod siwa;

/// Plugin state holding the mobile plugin handle (iOS only).
///
/// On iOS, `setup` stores the `PluginHandle` returned by
/// `register_ios_plugin` so that `start_auth` can delegate to the Swift
/// `AsWebAuthPlugin` via `run_mobile_plugin`.
///
/// On non-iOS targets this type is not used: macOS keeps no state (the
/// session lives entirely inside one `start_auth` call), and other platforms
/// return an error without touching plugin state.
#[cfg(target_os = "ios")]
pub struct AsWebAuthState<R: Runtime> {
    pub(crate) handle: PluginHandle<R>,
}

/// Initializes the plugin.
///
/// On iOS, registers the Swift `AsWebAuthPlugin` via
/// `api.register_ios_plugin()` and stores the handle in plugin state.
/// On macOS, no setup is needed — `start_auth` builds a fresh
/// `ASWebAuthenticationSession` per call on the main thread.
/// On other targets the plugin is a no-op — `start_auth` returns an
/// error because the frontend only invokes it on Apple platforms.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("aswebauth")
        .setup(|app, api| {
            #[cfg(target_os = "ios")]
            {
                let handle = api.register_ios_plugin(init_plugin_aswebauth)?;
                app.manage(AsWebAuthState::<R> { handle });
            }

            // macOS and other non-iOS targets: nothing to register.
            #[cfg(not(target_os = "ios"))]
            {
                let _ = (app, api);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_auth,
            commands::sign_in_with_apple
        ])
        .build()
}
