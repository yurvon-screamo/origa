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
//! WebGPU detection: We call `navigator.gpu.requestAdapter()` at runtime.
//! Just checking for the existence of `navigator.gpu` is insufficient —
//! Playwright headless Chromium exposes the property but `requestAdapter()`
//! returns null when no GPU is available. Registering the WebGPU EP in that
//! state causes session creation to download the 25 MB JSEP wasm bundle and
//! then hang. By actually requesting an adapter we get a definitive answer.

use crate::domain::OrigaError;
use futures::lock::Mutex;
use std::sync::OnceLock;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

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
    let webgpu_active = webgpu_adapter_available().await;

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

/// Probes for a real WebGPU adapter by calling `navigator.gpu.requestAdapter()`.
///
/// This is async because `requestAdapter()` returns a `Promise<GPUAdapter | null>`.
/// In headless Chromium (CI) the promise resolves with `null` — no GPU — so
/// we correctly fall back to CPU. On real user devices the adapter is non-null.
///
/// Checking only for `navigator.gpu` property existence is **insufficient**:
/// Playwright's bundled Chromium exposes the property even without a GPU.
async fn webgpu_adapter_available() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let navigator = window.navigator();

    let gpu = js_sys::Reflect::get(&navigator, &js_sys::JsString::from("gpu")).ok();
    let Some(gpu) = gpu else {
        return false;
    };
    if gpu.is_null() || gpu.is_undefined() {
        return false;
    }

    let request_adapter =
        js_sys::Reflect::get(&gpu, &js_sys::JsString::from("requestAdapter")).ok();
    let Some(request_adapter) = request_adapter else {
        return false;
    };

    let Ok(request_fn) = request_adapter.dyn_into::<js_sys::Function>() else {
        return false;
    };

    let adapter_result = request_fn.call0(&gpu);
    let Ok(adapter_promise_val) = adapter_result else {
        return false;
    };

    let Ok(adapter_promise) = adapter_promise_val.dyn_into::<js_sys::Promise>() else {
        return false;
    };

    match JsFuture::from(adapter_promise).await {
        Ok(adapter) => !adapter.is_null() && !adapter.is_undefined(),
        Err(e) => {
            tracing::warn!(error = ?e, "navigator.gpu.requestAdapter() rejected");
            false
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_outcome_is_copy() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<InitOutcome>();
    }
}
