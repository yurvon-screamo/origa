// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Arguments for the `start_auth` command.
///
/// Serialized from Rust to Swift via `run_mobile_plugin` as
/// `{ "url": "...", "callbackScheme": "origa" }`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(not(target_os = "ios"), expect(dead_code))]
pub struct StartAuthArgs {
    /// Full OAuth provider URL (PKCE challenge included).
    pub url: String,
    /// Custom URL scheme to intercept (e.g. "origa").
    #[serde(rename = "callbackScheme")]
    pub callback_scheme: String,
}

/// Successful response from `start_auth`.
///
/// Deserialized from Swift's `invoke.resolve(["url": ...])` via
/// `run_mobile_plugin`, then serialized to the frontend as
/// `{ "url": "origa://auth/callback?code=..." }`.
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthResult {
    /// The full callback URL intercepted by ASWebAuthenticationSession.
    pub url: String,
}

/// Successful response from `sign_in_with_apple` (native Sign in with Apple
/// via `ASAuthorizationController`).
///
/// Serialized from Rust to Swift via `run_mobile_plugin` and to the frontend
/// with camelCase keys. `nonce` is the RAW client nonce: its SHA-256 hash was
/// handed to the authorization request, and the login endpoint re-hashes the
/// raw value to match it against the identity token's `nonce` claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleCredential {
    /// The identity token JWT (`ASAuthorizationAppleIDCredential.identityToken`).
    #[serde(rename = "identityToken")]
    pub identity_token: String,
    /// The raw client-generated nonce.
    pub nonce: String,
}

/// iOS: delegates to the Swift `AsWebAuthPlugin.startAuth` via
/// `PluginHandle::run_mobile_plugin`. The Swift completion handler resolves
/// with `{ "url": "origa://auth/callback?code=..." }`.
#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn start_auth<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    url: String,
    callback_scheme: String,
) -> Result<AuthResult, String> {
    use crate::AsWebAuthState;
    use tauri::Manager;

    let state = app
        .try_state::<AsWebAuthState<R>>()
        .ok_or("aswebauth plugin not initialized")?;

    let result: AuthResult = state
        .handle
        .run_mobile_plugin(
            "startAuth",
            StartAuthArgs {
                url,
                callback_scheme,
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(result)
}

/// macOS: native `ASWebAuthenticationSession` via objc2. The session must be
/// created and started on the main thread; the async command dispatches there
/// and awaits the completion through a oneshot channel.
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn start_auth<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    url: String,
    callback_scheme: String,
) -> Result<AuthResult, String> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<AuthResult, String>>();

    let app_for_session = app.clone();
    let mut tx_for_session = Some(tx);
    app.run_on_main_thread(move || {
        // `start_session` owns the sender and guarantees exactly one send:
        // from the completion handler, or synchronously with the real setup
        // failure cause when the session cannot even be created.
        if let Some(tx) = tx_for_session.take() {
            crate::macos::start_session(&app_for_session, &url, &callback_scheme, tx);
        }
    })
    .map_err(|e| format!("failed to dispatch onto the main thread: {e}"))?;

    // The completion handler fires when the user finishes the flow — seconds
    // later. A blocking recv on a dedicated worker thread is the cheapest
    // bridge to async here (same pattern as device_ai_commands).
    let received = tauri::async_runtime::spawn_blocking(move || rx.recv())
        .await
        .map_err(|e| format!("authentication worker failed: {e}"))?;

    received.map_err(|_| "authentication session dropped before completing".to_string())?
}

/// Other platforms stub — always returns an error. Unreachable because the
/// frontend only invokes the command on Apple platforms and falls back to the
/// opener flow elsewhere.
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[tauri::command]
pub async fn start_auth(_url: String, _callback_scheme: String) -> Result<AuthResult, String> {
    Err("ASWebAuthenticationSession is only available on Apple platforms".to_string())
}

#[cfg(target_os = "ios")]
use tauri::Runtime;

/// iOS: delegates to the Swift `AsWebAuthPlugin.signInWithApple` via
/// `PluginHandle::run_mobile_plugin`. The Swift delegate resolves with
/// `{ "identityToken": ..., "nonce": ... }`.
#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn sign_in_with_apple<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<AppleCredential, String> {
    use crate::AsWebAuthState;
    use tauri::Manager;

    let state = app
        .try_state::<AsWebAuthState<R>>()
        .ok_or("aswebauth plugin not initialized")?;

    let result: AppleCredential = state
        .handle
        .run_mobile_plugin::<AppleCredential>("signInWithApple", ())
        .map_err(|e| e.to_string())?;

    Ok(result)
}

/// macOS: native Sign in with Apple via `ASAuthorizationController`
/// (`siwa.rs`). The controller must be created and started on the main
/// thread; the async command dispatches there and awaits the delegate
/// callback through an mpsc channel.
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn sign_in_with_apple<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<AppleCredential, String> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<AppleCredential, String>>();

    let app_for_sheet = app.clone();
    let mut tx_for_sheet = Some(tx);
    app.run_on_main_thread(move || {
        // `start_native_sign_in` owns the sender and guarantees exactly one
        // send: from the delegate callback, or synchronously with the real
        // failure cause when the sheet cannot even be created.
        if let Some(tx) = tx_for_sheet.take() {
            crate::siwa::start_native_sign_in(&app_for_sheet, tx);
        }
    })
    .map_err(|e| format!("failed to dispatch onto the main thread: {e}"))?;

    // The delegate callback fires when the user finishes the flow — seconds
    // later. A blocking recv on a dedicated worker thread is the cheapest
    // bridge to async here (same pattern as `start_auth`).
    let received = tauri::async_runtime::spawn_blocking(move || rx.recv())
        .await
        .map_err(|e| format!("apple sign-in worker failed: {e}"))?;

    received.map_err(|_| "apple sign-in sheet dropped before completing".to_string())?
}

/// Other platforms stub — always returns an error. Unreachable because the
/// frontend only invokes the command on Apple platforms.
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[tauri::command]
pub async fn sign_in_with_apple() -> Result<AppleCredential, String> {
    Err("Sign in with Apple is only available on Apple platforms".to_string())
}
