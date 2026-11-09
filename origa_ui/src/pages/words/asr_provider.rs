//! device-ai native ASR paths.
//!
//! File-based recognition requires the macOS-only `device_ai_recognize_file`
//! Tauri command (the JS plugin surface accepts only live-microphone input).
//! Live recognition goes through the plugin's `speech_recognize` command.
//! Both return `None` when unavailable so callers fall back to Whisper WASM.

use crate::core::device_ai::{self, Feature};
use tracing::{error, info};

const JA_JP: &str = "ja-JP";

/// Recognize speech in a base64-encoded audio buffer via native device-ai
/// (macOS desktop). Returns `Some(text)` on success, or `None` when
/// unavailable/failed — callers fall back to Whisper WASM.
///
/// WASM-only: invoked solely from the WASM transcription path.
#[cfg(target_arch = "wasm32")]
pub(super) async fn recognize_file_via_device_ai(base64_audio: &str) -> Option<String> {
    use crate::core::tauri;
    use leptos::wasm_bindgen::JsValue;

    const CMD_RECOGNIZE_FILE: &str = "device_ai_recognize_file";

    if !device_ai::available(Feature::SpeechRecognition).await {
        info!("ASR (file): native device-ai unavailable, using Whisper WASM fallback");
        return None;
    }
    info!("ASR (file): using native device-ai recognition");

    let inner = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &inner,
        &JsValue::from_str("base64"),
        &JsValue::from_str(base64_audio),
    );
    let _ = js_sys::Reflect::set(
        &inner,
        &JsValue::from_str("language"),
        &JsValue::from_str(JA_JP),
    );

    let payload = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&payload, &JsValue::from_str("args"), &inner);

    match tauri::invoke_with_args(CMD_RECOGNIZE_FILE, &payload).await {
        Ok(value) => js_sys::Reflect::get(&value, &JsValue::from_str("text"))
            .ok()
            .and_then(|t| t.as_string()),
        Err(e) => {
            // Same reachability as the OCR escalation (ocr_device_ai.rs):
            // only fired after capabilities reported the feature available,
            // so this is a genuine degradation of the native path — escalate
            // to a Sentry event via the tracing bridge; the error string
            // carries the plugin's structured code.
            error!("device-ai file ASR failed, falling back to Whisper: {e}");
            None
        },
    }
}

/// Error code the plugin reports when a live session ended without any
/// recognized speech (bounded no-speech budget, see the fork's
/// `speech_live_ctrl.rs`).
const NO_SPEECH_CODE: &str = "[NO_SPEECH_DETECTED]";

/// Perform one-shot live-microphone recognition via native device-ai.
/// Returns `Some(text)` on success, or `None` when unavailable/failed.
pub(super) async fn recognize_live_via_device_ai() -> Option<String> {
    if !device_ai::available(Feature::SpeechRecognition).await {
        info!("ASR (live): native device-ai unavailable, using Whisper WASM fallback");
        return None;
    }
    info!("ASR (live): using native device-ai recognition");

    match device_ai::recognize_live(JA_JP).await {
        Ok(result) => Some(result.text),
        Err(e) => {
            // "No speech" is an expected user outcome, not a failure of the
            // native path — returning an empty string routes the caller to
            // its no-speech message instead of disabling the button.
            if e.contains(NO_SPEECH_CODE) {
                info!("ASR (live): no speech detected");
                return Some(String::new());
            }
            // Same reachability as the OCR escalation (ocr_device_ai.rs):
            // native path failed after caps=available — a genuine
            // degradation, escalate to a Sentry event via the tracing
            // bridge. (NO_SPEECH above is an expected user outcome and
            // stays a breadcrumb-level info.)
            error!("device-ai live ASR failed: {e}");
            None
        },
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::NO_SPEECH_CODE;

    /// The marker must match how `describe_rejection` formats plugin errors
    /// (see `core/device_ai/invoke.rs`) — both sides of the contract are
    /// plain strings, so a drift would silently route "no speech" to the
    /// "unavailable" fallback.
    #[test]
    fn no_speech_marker_matches_invoke_error_format() {
        let rejection = format!("plugin command rejected: {NO_SPEECH_CODE} No speech detected");
        assert!(rejection.contains(NO_SPEECH_CODE));

        // Unrelated failures must not match the marker.
        let timeout = "plugin command timed out after 45s".to_string();
        assert!(!timeout.contains(NO_SPEECH_CODE));
        let generic = "plugin command rejected: [SPEECH_RECOGNITION_FAILED] boom".to_string();
        assert!(!generic.contains(NO_SPEECH_CODE));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    //! Pins the ASR invoke-failure escalations (task 20261104051142-inn7qka,
    //! review round 1 Common-1): a native-path failure after caps=available
    //! must log at `error` level (a Sentry event) — both the file and the
    //! live path — while the no-speech outcome stays `info` (a user outcome,
    //! not a degradation). Levels themselves are the assertion targets: the
    //! returned values (`None` / `Some("")`) are identical across levels.

    use super::*;
    use crate::core::log_capture;
    use crate::core::tauri_mock::{TauriMock, all_available_caps, rejected_with};
    use tracing::Level;

    /// Caps resolve available; every downstream command rejects with the
    /// given code/message (the exact rejection per test).
    fn plugin_mock(reject_code: &str, reject_message: &str) -> TauriMock {
        let code = reject_code.to_string();
        let message = reject_message.to_string();
        TauriMock::install(move |command, _args| {
            let command = command.as_string().unwrap_or_default();
            if command.ends_with("get_capabilities") {
                all_available_caps()
            } else {
                rejected_with(&code, &message)
            }
        })
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn file_asr_invoke_failure_emits_error_event_and_falls_back() {
        log_capture::install();
        device_ai::reset_cached_capabilities();
        let _mock = plugin_mock("SPEECH_RECOGNITION_FAILED", "file path broken");

        let result = recognize_file_via_device_ai("YXVkaW8=").await;

        assert!(
            result.is_none(),
            "failed native file ASR must fall back (return None)"
        );
        let events = log_capture::take_events()
            .into_iter()
            .filter(|(_, message)| message.contains("device-ai file ASR failed"))
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 1, "got: {events:?}");
        assert_eq!(
            events[0].0,
            Level::ERROR,
            "native file-ASR failure must be an error (Sentry event), got: {:?}",
            events[0]
        );
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn live_asr_invoke_failure_emits_error_event_and_falls_back() {
        log_capture::install();
        device_ai::reset_cached_capabilities();
        let _mock = plugin_mock("SPEECH_RECOGNITION_FAILED", "live path broken");

        let result = recognize_live_via_device_ai().await;

        assert!(
            result.is_none(),
            "failed native live ASR must fall back (return None)"
        );
        let events = log_capture::take_events()
            .into_iter()
            .filter(|(_, message)| message.contains("device-ai live ASR failed"))
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 1, "got: {events:?}");
        assert_eq!(
            events[0].0,
            Level::ERROR,
            "native live-ASR failure must be an error (Sentry event), got: {:?}",
            events[0]
        );
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn live_asr_no_speech_stays_info_and_routes_to_empty_string() {
        log_capture::install();
        device_ai::reset_cached_capabilities();
        let _mock = plugin_mock("NO_SPEECH_DETECTED", "No speech detected");

        let result = recognize_live_via_device_ai().await;

        assert_eq!(
            result.as_deref(),
            Some(""),
            "no-speech must route to the caller's empty-string branch, not a fallback"
        );
        // One snapshot, two filters — take_events() clears the buffer, so a
        // second call would see an empty log.
        let all = log_capture::take_events();
        let failures: Vec<_> = all
            .iter()
            .filter(|(_, message)| message.contains("device-ai live ASR failed"))
            .cloned()
            .collect();
        assert!(
            failures.is_empty(),
            "no-speech must not produce failure events, got: {failures:?}"
        );
        let no_speech: Vec<_> = all
            .iter()
            .filter(|(_, message)| message.contains("no speech detected"))
            .cloned()
            .collect();
        assert_eq!(no_speech.len(), 1, "all events: {all:?}");
        assert_eq!(
            no_speech[0].0,
            Level::INFO,
            "no-speech is an expected user outcome — it must stay info, got: {:?}",
            no_speech[0]
        );
    }
}
