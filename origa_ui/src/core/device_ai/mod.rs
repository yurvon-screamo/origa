//! High-level device-ai provider: capabilities-first routing decisions plus
//! cached access to the plugin's native ASR/OCR.
//!
//! Routing contract:
//! - Web (no Tauri) → the plugin is unreachable; [`available`] returns `false`
//!   for every feature and callers use the WASM fallback.
//! - Tauri on Windows/Linux → the plugin is not compiled in; invoking it
//!   fails, capabilities stay unavailable, fallback is used.
//! - Tauri on macOS/iOS/Android → capabilities are queried once, cached, and
//!   each feature routes to device-ai when reported available, otherwise to
//!   the fallback stack.
//!
//! All plugin access goes through [`invoke`], which enforces a per-call
//! timeout so a hung native call falls back instead of freezing the UI.

pub mod contracts;
#[cfg(all(target_arch = "wasm32", test))]
mod degradation_wasm_tests;
mod invoke;
#[cfg(all(target_arch = "wasm32", test))]
mod invoke_wasm_tests;

use std::cell::RefCell;

use contracts::{Capabilities, RecognitionResult, TextRecognitionResult};

use crate::core::tauri;

/// Feature identifiers used by routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    SpeechRecognition,
    TextRecognition,
}

impl Feature {
    fn status(self, caps: &Capabilities) -> bool {
        match self {
            Self::SpeechRecognition => caps.speech_recognition.available,
            Self::TextRecognition => caps.text_recognition.available,
        }
    }
}

thread_local! {
    /// Capabilities cache. `None` = not yet queried; `Some(caps)` = queried
    /// (possibly all-unavailable). Queried once per session to avoid repeated
    /// plugin round-trips on every OCR/ASR call.
    static CACHED_CAPABILITIES: RefCell<Option<Capabilities>> = const { RefCell::new(None) };
}

/// Test hook: clears the capabilities cache so each test starts clean and
/// stays order-independent.
#[cfg(all(target_arch = "wasm32", test))]
pub(crate) fn reset_cached_capabilities() {
    CACHED_CAPABILITIES.with(|c| *c.borrow_mut() = None);
}

/// Returns `true` if the device-ai plugin could be present at all.
///
/// Cheap, synchronous, allocation-free — the first check before any async
/// plugin round-trip. This is a runtime check, not a compile-time `cfg`:
/// the frontend is one WASM binary shared across every Tauri host, so
/// `target_os` is always `unknown` here. The actual per-platform presence
/// (plugin registered on macOS/iOS/Android, absent on Windows/Linux) is
/// resolved by the capabilities query — an unregistered plugin rejects the
/// `get_capabilities` invoke and collapses to "unavailable".
pub fn plugin_compiled_in() -> bool {
    tauri::is_tauri()
}

/// Reports whether `feature` is available via device-ai on this platform.
///
/// Triggers a one-shot capabilities query on first use, then reads the cache.
/// Any failure (plugin absent, invoke error, timeout) collapses to
/// "unavailable" so callers transparently use the fallback stack.
pub async fn available(feature: Feature) -> bool {
    if !plugin_compiled_in() {
        tracing::debug!("device-ai: {feature:?} unavailable — not running in a Tauri WebView");
        return false;
    }
    let caps = match cached_or_query().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "device-ai: {feature:?} — capabilities query failed ({e}), using fallback"
            );
            return false;
        },
    };
    let status = feature.status(&caps);
    tracing::debug!("device-ai: {feature:?} available = {status}");
    status
}

/// Pure routing predicate: does this platform have a device-ai plugin backend
/// that a capabilities query could reach at all? Windows/Linux/web have no
/// plugin by design (`expects_native_ai` is the canonical platform matrix) —
/// a query there is doomed and only produces ACL-reject noise (uwuwu#1098),
/// so [`cached_or_query`] collapses to all-unavailable without the
/// round-trip. Kept as a named function (not an inline call) so the routing
/// contract stays unit-testable without a webview.
fn capabilities_query_expected(platform: &str) -> bool {
    crate::core::capabilities::expects_native_ai(platform)
}

/// Resolves capabilities from the cache, querying the plugin on first access.
///
/// Concurrent first callers share one in-flight query (single-flight lives in
/// [`invoke::get_capabilities`]), so everyone receives the same real result —
/// nobody collapses to "unavailable" merely because a query is already
/// running. This matters because callers are one-shot (e.g. the live-recorder
/// mount effect): without sharing, the loser would never re-check (#570).
///
/// Only a *successful* query is cached — including an honest "all features
/// unavailable". A transient failure (plugin not yet ready, timeout, decode
/// error) is returned as `all_unavailable` for the current call but is NOT
/// cached, so the next call retries instead of permanently disabling device-ai
/// for the whole session.
async fn cached_or_query() -> Result<Capabilities, String> {
    if let Some(caps) = CACHED_CAPABILITIES.with(|c| c.borrow().clone()) {
        return Ok(caps);
    }

    // Platforms without the compiled-in plugin (Windows/Linux — uwuwu#1098)
    // never pass the ACL: querying them only produces the "Command … not
    // allowed by ACL" warn on every boot. Collapse to all-unavailable
    // without the doomed round-trip; the fallback routing below is their
    // normal path, not a degradation.
    let platform = crate::core::platform::platform_name();
    if !capabilities_query_expected(&platform) {
        tracing::debug!(
            "device-ai: no plugin backend on {platform} — capabilities unavailable, fallbacks only"
        );
        return Ok(Capabilities::all_unavailable());
    }

    match invoke::get_capabilities().await {
        Ok(caps) => {
            CACHED_CAPABILITIES.with(|c| *c.borrow_mut() = Some(caps.clone()));
            tracing::info!(
                "device-ai: native capabilities — speech_recognition={}, text_recognition={}",
                caps.speech_recognition.available,
                caps.text_recognition.available,
            );
            Ok(caps)
        },
        Err(e) => {
            // Unreachable for plugin-less platforms (skipped above), so every
            // Err here is a capabilities failure on a platform that expects
            // the native path — a degradation, not routine fallback routing:
            // escalate to an error so it surfaces as a Sentry event (warn
            // only lands as a breadcrumb, which is invisible without a later
            // capture).
            tracing::error!(
                "device-ai: capabilities query failed ({e}) — native path degraded on a \
                 platform that expects it"
            );
            Ok(Capabilities::all_unavailable())
        },
    }
}

/// Recognize text in a base64-encoded image via native OCR. Returns `Err` if
/// the plugin is unavailable or the call fails — callers fall back to WASM.
pub async fn recognize_text(base64: &str) -> Result<TextRecognitionResult, String> {
    invoke::recognize_text(base64).await
}

/// One-shot live-microphone recognition. Returns `Err` if unavailable —
/// callers fall back to recording + Whisper WASM.
pub async fn recognize_live(language: &str) -> Result<RecognitionResult, String> {
    invoke::recognize_live(language).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_status_reads_available_flag() {
        let mut caps = Capabilities::all_unavailable();
        caps.speech_recognition.available = true;

        assert!(Feature::SpeechRecognition.status(&caps));
        assert!(!Feature::TextRecognition.status(&caps));
    }

    #[test]
    fn all_unavailable_reports_false_for_every_feature() {
        let caps = Capabilities::all_unavailable();

        assert!(!Feature::SpeechRecognition.status(&caps));
        assert!(!Feature::TextRecognition.status(&caps));
    }

    /// uwuwu#1098: the capabilities query is expected only on platforms with
    /// a plugin backend; on Windows/Linux the query is skipped entirely (no
    /// doomed ACL round-trip), on web there is no Tauri shell.
    #[test]
    fn capabilities_query_expected_only_on_plugin_platforms() {
        for platform in ["macos", "ios", "android"] {
            assert!(
                capabilities_query_expected(platform),
                "{platform} hosts the plugin backend — query expected"
            );
        }
        for platform in ["linux", "windows"] {
            assert!(
                !capabilities_query_expected(platform),
                "{platform} has no plugin backend — the query must be skipped"
            );
        }
        assert!(
            !capabilities_query_expected("webview-unknown"),
            "non-plugin platforms must skip the query"
        );
    }
}
