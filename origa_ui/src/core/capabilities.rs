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

    /// Emits a Sentry event per unavailable feature — one event each, no
    /// noise. This turns silent degradation observable for all users (the
    /// #570 "Observatory" recommendation). `tracing::error!` rides the
    /// sentry-tracing bridge (console.error → captureConsoleIntegration) as
    /// an actual event; breadcrumbs alone never surface without a later
    /// capture, which is exactly how the Android native-OCR degradation
    /// stayed invisible (task 20261104051142-inn7qka).
    fn report_degradations(&self) {
        let unavailable = |name: &str, expected_on: &[&str]| {
            if expected_on.contains(&self.platform.as_str()) {
                tracing::error!(
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

/// Whether the device-ai plugin is expected to provide native ASR/OCR on
/// this platform. Expected platforms are the ones with a registered plugin
/// backend (macOS/iOS/Android, see the [`device_ai`] routing contract);
/// there a native-path failure is a *degradation* — on Windows/Linux the
/// plugin is absent by design and on web there is no Tauri shell, so their
/// fallback routing is the normal path, not a degradation.
pub fn expects_native_ai(platform: &str) -> bool {
    matches!(platform, "android" | "ios" | "macos")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_ai_expected_only_on_plugin_platforms() {
        for expected in ["android", "ios", "macos"] {
            assert!(expects_native_ai(expected), "{expected} must expect native");
        }
        for other in ["windows", "linux", "web", "unknown"] {
            assert!(!expects_native_ai(other), "{other} must not expect native");
        }
    }
}
