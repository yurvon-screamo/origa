//! Centralized ort-web initialization with runtime WebGPU detection.
//!
//! ort-web requires `ort::set_api(...)` to be called exactly once before any
//! other `ort` API. This module provides an async-safe initializer that every
//! WASM model loader (`WhisperTranscriber::new`, `DeimDetector::new`,
//! `ParseqRecognizer::new`) calls via [`ensure`]. The first caller performs
//! the actual initialization; subsequent callers receive the memoized result.
//!
//! Concurrency: WASM is single-threaded (cooperative async), but two futures
//! can still interleave around `.await`. A `futures::lock::Mutex` guard
//! serializes the initialization; `std::sync::OnceLock` (works on WASM)
//! memoizes the result. Errors are stored too — if initialization fails once,
//! subsequent calls return the same error without retrying.
//!
//! WebGPU detection: We probe `navigator.gpu` at runtime. Registering the
//! WebGPU execution provider on a runtime that has no GPU adapter (headless
//! CI Chromium, old WebView) blocks session creation indefinitely, so we
//! load `FEATURE_NONE` (CPU-only) when WebGPU is unavailable. On real user
//! devices (Windows WebView2 113+, macOS/iOS 26+, Android WebView 121+) the
//! probe returns true and we load `FEATURE_WEBGPU`. WebGL is intentionally
//! not included — `ort` crate has no `ep::WebGL` struct, so the WebGL bundle
//! would be downloaded but never registered.

use crate::domain::OrigaError;
use futures::lock::Mutex;
use std::sync::OnceLock;

/// Outcome of one-time ort-web initialization.
///
/// `webgpu_active == true` means sessions may register `ep::WebGPU`;
/// `false` means CPU-only and registration must be skipped (the EP isn't
/// loaded in the fetched build).
#[derive(Clone, Copy, Debug)]
pub struct InitOutcome {
    pub webgpu_active: bool,
}

type Stored = Result<InitOutcome, OrigaError>;

static INIT: OnceLock<Stored> = OnceLock::new();
static INIT_GUARD: Mutex<()> = Mutex::new(());

/// Ensures ort-web is initialized.
///
/// Idempotent and async-safe: the first caller triggers `ort_web::api(...)`
/// and `ort::set_api`; concurrent and later callers block on the guard and
/// then receive the stored outcome.
pub async fn ensure() -> Result<InitOutcome, OrigaError> {
    if let Some(stored) = INIT.get() {
        return stored.clone();
    }

    let _guard = INIT_GUARD.lock().await;

    if let Some(stored) = INIT.get() {
        return stored.clone();
    }

    let result = init_inner().await;
    let _ = INIT.set(result.clone());
    result
}

async fn init_inner() -> Result<InitOutcome, OrigaError> {
    let webgpu_active = webgpu_available();

    let feature = if webgpu_active {
        tracing::info!("WebGPU adapter detected, initializing ort-web with FEATURE_WEBGPU");
        ort_web::FEATURE_WEBGPU
    } else {
        tracing::info!(
            "WebGPU adapter unavailable, initializing ort-web with FEATURE_NONE (CPU-only)"
        );
        ort_web::FEATURE_NONE
    };

    let api = ort_web::api(feature)
        .await
        .map_err(|e| OrigaError::OcrError {
            reason: format!("ort_web::api failed: {e:?}"),
        })?;

    ort::set_api(api);

    tracing::info!(webgpu_active, "ort-web initialized");
    Ok(InitOutcome { webgpu_active })
}

/// Probes the host environment for a WebGPU adapter.
///
/// Checks for `navigator.gpu` existence via `Reflect::has` (cheap, no async
/// adapter request). In headless Chromium without `--enable-unsafe-webgpu`
/// and GPU passthrough this returns `false`, which keeps CI green; on real
/// user devices it returns `true` when the WebView supports WebGPU.
fn webgpu_available() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let navigator = window.navigator();
    js_sys::Reflect::has(&navigator, &js_sys::JsString::from("gpu")).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_outcome_is_copy() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<InitOutcome>();
    }

    #[test]
    fn webgpu_available_returns_bool_in_test_env() {
        let _ = webgpu_available();
    }
}
