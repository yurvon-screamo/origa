//! device-ai native OCR path.
//!
//! When the native text-recognition capability is available, OCR runs through
//! the platform's on-device API (no model download) and returns immediately.
//! Any failure (capability unavailable, invoke error, timeout) yields `None`
//! so the caller transparently falls back to the WASM NDLOCR pipeline.

use crate::core::device_ai::{self, Feature};
use crate::ui_components::{OcrLoadingStage, OcrLoadingState};
use leptos::prelude::*;
use tracing::{error, info};

/// Attempts native OCR on a base64-encoded image. Returns `Some(text)` on
/// success, or `None` when native OCR is unavailable or fails — signalling
/// the caller to use the WASM fallback.
pub(super) async fn recognize_via_device_ai(
    base64_data: &str,
    loading_state: &OcrLoadingState,
) -> Option<String> {
    if !device_ai::available(Feature::TextRecognition).await {
        info!("OCR: native device-ai unavailable, using WASM NDLOCR fallback");
        return None;
    }
    info!("OCR: using native device-ai text recognition");

    // No model download or initialization stage for native OCR — jump straight
    // to recognition so the UI reflects the actual work.
    loading_state.stage.set(OcrLoadingStage::Recognizing);

    match device_ai::recognize_text(base64_data).await {
        Ok(result) => Some(result.text),
        Err(e) => {
            // Only reachable when capabilities reported the feature
            // available, so this is a genuine degradation of the native path
            // (plugin present, call failed) — escalate to a Sentry event via
            // the tracing bridge; the error string carries the plugin's
            // structured code ([FEATURE_NOT_AVAILABLE], [INVALID_IMAGE], …).
            error!("device-ai OCR failed, falling back to WASM: {e}");
            None
        },
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    //! Pins the OCR invoke-failure escalation (task 20261104051142-inn7qka):
    //! a native-path failure after caps=available must log at `error` level
    //! (a Sentry event) and return `None` (WASM fallback). A downgrade back
    //! to `warn` would make the degradation invisible in Sentry — the level
    //! itself is the assertion target.

    use super::*;
    use crate::core::log_capture;
    use crate::core::tauri_mock::{TauriMock, all_available_caps, rejected_with};
    use tracing::Level;

    fn plugin_mock_with_failing_vision() -> TauriMock {
        TauriMock::install(|command, _args| {
            let command = command.as_string().unwrap_or_default();
            if command.ends_with("get_capabilities") {
                all_available_caps()
            } else if command.ends_with("vision_recognize_text") {
                rejected_with("FEATURE_NOT_AVAILABLE", "recognizer unavailable")
            } else {
                rejected_with("UNEXPECTED_COMMAND", &command)
            }
        })
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn ocr_invoke_failure_emits_error_event_and_falls_back() {
        log_capture::install();
        device_ai::reset_cached_capabilities();
        let _mock = plugin_mock_with_failing_vision();

        let loading = OcrLoadingState::new();
        let result = recognize_via_device_ai("aGVsbG8=", &loading).await;

        assert!(
            result.is_none(),
            "failed native OCR must fall back (return None)"
        );
        let events = log_capture::take_events()
            .into_iter()
            .filter(|(_, message)| message.contains("device-ai OCR failed"))
            .collect::<Vec<_>>();
        assert_eq!(
            events.len(),
            1,
            "exactly one OCR failure event, got: {events:?}"
        );
        assert_eq!(
            events[0].0,
            Level::ERROR,
            "native OCR failure must be an error (Sentry event), got: {:?}",
            events[0]
        );
        assert!(
            events[0].1.contains("[FEATURE_NOT_AVAILABLE]"),
            "the structured plugin code must ride in the event body: {:?}",
            events[0]
        );
    }
}
