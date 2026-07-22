//! Centralized ort-web initialization.
//!
//! ort-web requires `ort::set_api(...)` to be called exactly once before any
//! other `ort` API. This module provides an async-safe initializer that every
//! WASM model loader (`WhisperTranscriber::new`, `DeimDetector::new`,
//! `ParseqRecognizer::new`) calls via [`ensure`]. The first caller performs
//! the actual initialization; subsequent callers receive the memoized result.
//!
//! Concurrency: WASM is single-threaded (cooperative async), but two futures
//! can still interleave around `.await`. A `futures::lock::Mutex` guard
//! serializes the initialization; `std::cell::OnceCell` (sufficient for
//! single-threaded WASM) memoizes the result. Errors are stored too — if
//! initialization fails once, subsequent calls return the same error without
//! retrying.
//!
//! Loaded build: `FEATURE_WEBGPU` only. WebGL is intentionally not included —
//! `ort` crate has no `ep::WebGL` struct, so the WebGL bundle would be
//! downloaded but never registered. ort runtime falls back to CPU EP
//! automatically when WebGPU is unavailable in the WebView.

use crate::domain::OrigaError;
use futures::lock::Mutex;
use ort_web::FEATURE_WEBGPU;
use std::sync::OnceLock;

type InitResult = Result<(), OrigaError>;

static INIT: OnceLock<InitResult> = OnceLock::new();
static INIT_GUARD: Mutex<()> = Mutex::new(());

/// Ensures ort-web is initialized with the WebGPU execution provider.
///
/// Idempotent and async-safe: the first caller triggers
/// `ort_web::api(FEATURE_WEBGPU)` and `ort::set_api`; concurrent and later
/// callers block on the guard and then receive the stored result.
pub async fn ensure() -> Result<(), OrigaError> {
    if let Some(result) = INIT.get() {
        return result.clone();
    }

    let _guard = INIT_GUARD.lock().await;

    if let Some(result) = INIT.get() {
        return result.clone();
    }

    let result = init_inner().await;
    let _ = INIT.set(result.clone());
    result
}

async fn init_inner() -> Result<(), OrigaError> {
    tracing::info!("Initializing ort-web with FEATURE_WEBGPU");

    let api = ort_web::api(FEATURE_WEBGPU)
        .await
        .map_err(|e| OrigaError::OcrError {
            reason: format!("ort_web::api(FEATURE_WEBGPU) failed: {e:?}"),
        })?;

    ort::set_api(api);

    tracing::info!("ort-web initialized, WebGPU EP available");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_webgpu_flag_value() {
        assert_eq!(FEATURE_WEBGPU, 0b10);
    }
}
