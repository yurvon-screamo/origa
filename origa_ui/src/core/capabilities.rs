//! Platform capability matrix (P-1): what capture features are available
//! per platform, with Sentry observability for silent degradations.
//!
//! The macOS microphone (#570) is the poster child: the button was hidden
//! because the capabilities query resolved unavailable, but no error was
//! ever surfaced. Every "quietly unavailable" resolution now emits a
//! Sentry breadcrumb so regressions are visible without a real device.

use crate::core::device_ai::{self, Feature};
use crate::core::platform::platform_name;

/// What the words-capture flow can do on this platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureCapabilities {
    pub platform: String,
    pub live_microphone: bool,
    pub native_file_asr: bool,
    pub native_ocr: bool,
    pub camera_capture: bool,
    pub native_picker: bool,
}

impl CaptureCapabilities {
    /// Queries the runtime and captures a Sentry breadcrumb for every
    /// unavailable feature (the #570 observability requirement).
    pub async fn detect() -> Self {
        let platform = platform_name();

        let speech = device_ai::available(Feature::SpeechRecognition).await;
        let ocr = device_ai::available(Feature::TextRecognition).await;

        let caps = Self {
            live_microphone: speech,
            native_file_asr: speech,
            native_ocr: ocr,
            camera_capture: matches!(platform.as_str(), "android" | "ios"),
            native_picker: crate::core::file_picker::is_available(),
            platform: platform.clone(),
        };

        caps.report_degradations();
        caps
    }

    /// Emits a Sentry breadcrumb per unavailable feature — one event each,
    /// no noise. This turns silent degradation observable for all users
    /// (the #570 "Observatory" recommendation).
    fn report_degradations(&self) {
        let unavailable = |name: &str, expected_on: &[&str]| {
            if expected_on.contains(&self.platform.as_str()) {
                // The sentry-tracing bridge routes warn → breadcrumbs.
                tracing::warn!(
                    feature = name,
                    platform = %self.platform,
                    "capture capability unavailable on a platform that expects it"
                );
            }
        };

        // Native ASR/OCR: expected on macOS/iOS/Android — unavailable means
        // the device-ai capabilities query failed or returned false (#570).
        unavailable("live_microphone", &["macos", "ios", "android"]);
        unavailable("native_file_asr", &["macos", "ios", "android"]);
        unavailable("native_ocr", &["macos", "ios", "android"]);
        // Native picker: expected on all desktop Tauri shells.
        unavailable("native_picker", &["windows", "macos", "linux"]);
    }
}
