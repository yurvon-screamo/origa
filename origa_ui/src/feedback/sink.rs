//! Feedback submission transport.
//!
//! [`FeedbackSink`] abstracts where reports go. The production implementation
//! is [`SentryFeedbackSink`] (Sentry user-feedback API via the JS SDK loaded
//! by [`crate::sentry`]); a fallback transport (e.g. a TrailBase table) can be
//! added later without touching the modal or the entry points.
//!
//! ## Error contract (ADR-055)
//!
//! - `SentryUnavailable` is returned **only** when the compile-time
//!   `SENTRY_DSN_UI` is empty (dev/e2e builds). The modal shows an
//!   informational state ("available in release builds") — never an error.
//! - Any failure while the DSN *is* configured (SDK not yet loaded, method
//!   missing after a major-version drift of the loader script, transport
//!   rejection) is `SubmitFailed` — a real error state with a retry, because
//!   a release user must never see "release builds only" inside a release.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use js_sys::{Object, Reflect};
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use super::types::FeedbackReport;

/// Why a submission did not go through.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackSubmitError {
    /// Sentry is compiled out (empty `SENTRY_DSN_UI`) — informational, not an
    /// error. Maps to the "available in release builds" modal state.
    SentryUnavailable,
    /// The SDK was configured but the submission failed — retryable error.
    SubmitFailed(String),
}

/// Transport abstraction for feedback reports.
pub trait FeedbackSink {
    /// Submit a fully assembled report. `Err` never panics the caller; the
    /// modal maps the variants to its info/error states.
    fn submit(
        &self,
        report: &FeedbackReport,
    ) -> impl Future<Output = Result<(), FeedbackSubmitError>>;
}

/// Production sink: `window.Sentry.captureFeedback` (Sentry JS SDK v8+).
///
/// Follows the defensive JS-interop pattern of `sentry::capture_exception`:
/// every `Reflect` access is checked; a missing global/method degrades into
/// `SubmitFailed` instead of a panic.
pub struct SentryFeedbackSink;

impl SentryFeedbackSink {
    /// Compile-time DSN emptiness — the single legitimate trigger of the
    /// "available in release builds" info state (ADR-055).
    fn compiled_out() -> bool {
        crate::sentry::dsn_ui().is_empty()
    }

    fn sentry_global() -> Option<JsValue> {
        let window = web_sys::window()?;
        let value = Reflect::get(&window.into(), &JsValue::from_str("Sentry")).ok()?;
        if value.is_undefined() || value.is_null() {
            None
        } else {
            Some(value)
        }
    }
}

impl FeedbackSink for SentryFeedbackSink {
    async fn submit(&self, report: &FeedbackReport) -> Result<(), FeedbackSubmitError> {
        if Self::compiled_out() {
            return Err(FeedbackSubmitError::SentryUnavailable);
        }

        let sentry = Self::sentry_global().ok_or_else(|| {
            FeedbackSubmitError::SubmitFailed(
                "Sentry SDK global not available (loader not finished or failed)".to_string(),
            )
        })?;

        let capture = Reflect::get(&sentry, &JsValue::from_str("captureFeedback"))
            .map_err(|e| FeedbackSubmitError::SubmitFailed(format!("no captureFeedback: {e:?}")))?;
        let capture_fn = capture.dyn_ref::<js_sys::Function>().ok_or_else(|| {
            FeedbackSubmitError::SubmitFailed(
                "captureFeedback is not a function (SDK major-version drift?)".to_string(),
            )
        })?;

        // Argument 1: { message } (the only required field; name/email are
        // deliberately omitted — reports are anonymous).
        let payload = Object::new();
        Reflect::set(
            &payload,
            &JsValue::from_str("message"),
            &JsValue::from_str(&report.compose_message()),
        )
        .map_err(|e| FeedbackSubmitError::SubmitFailed(format!("payload build failed: {e:?}")))?;

        // Argument 2 hint: captureContext with filterable tags + structured
        // extra, so the feedback stream is triageable by category/source.
        let env = &report.environment;
        let tags = Object::new();
        for (key, value) in [
            ("feedback_category", report.category.as_str().to_string()),
            ("feedback_source", report.source.as_str().to_string()),
            ("app_version", env.app_version.to_string()),
            ("platform", env.platform.clone()),
            ("page", env.page.clone()),
            ("ui_language", env.ui_language.clone()),
        ] {
            let _ = Reflect::set(&tags, &JsValue::from_str(key), &JsValue::from_str(&value));
        }

        let subject = Object::new();
        let _ = Reflect::set(
            &subject,
            &JsValue::from_str("surface"),
            &JsValue::from_str(&report.subject.surface),
        );
        if let Some(reading) = report.subject.reading.as_deref() {
            let _ = Reflect::set(
                &subject,
                &JsValue::from_str("reading"),
                &JsValue::from_str(reading),
            );
        }
        if let Some(line) = report.subject.context_line.as_deref() {
            let _ = Reflect::set(
                &subject,
                &JsValue::from_str("context_line"),
                &JsValue::from_str(line),
            );
        }

        let extra = Object::new();
        let _ = Reflect::set(&extra, &JsValue::from_str("subject"), &subject);
        let _ = Reflect::set(
            &extra,
            &JsValue::from_str("form"),
            &JsValue::from_str("origa-inapp"),
        );

        let capture_context = Object::new();
        let _ = Reflect::set(&capture_context, &JsValue::from_str("tags"), &tags);
        let _ = Reflect::set(&capture_context, &JsValue::from_str("extra"), &extra);

        let hint = Object::new();
        let _ = Reflect::set(
            &hint,
            &JsValue::from_str("captureContext"),
            &capture_context,
        );

        let result = capture_fn.call2(&sentry, &payload, &hint).map_err(|e| {
            FeedbackSubmitError::SubmitFailed(format!("captureFeedback threw: {e:?}"))
        })?;

        // The SDK returns a promise; a rejection (quota, transport) must not
        // surface as success.
        if let Ok(promise) = result.dyn_into::<js_sys::Promise>() {
            if let Err(e) = JsFuture::from(promise).await {
                return Err(FeedbackSubmitError::SubmitFailed(format!(
                    "captureFeedback rejected: {e:?}"
                )));
            }
        }

        Ok(())
    }
}

/// Boxed submit future stored by the feedback context.
pub type BoxedSubmitFuture =
    Pin<Box<dyn Future<Output = Result<(), FeedbackSubmitError>> + 'static>>;

/// Type-erased submit callable (a one-method sink): the context stores this so
/// wasm tests can inject a fake transport without generics on components.
/// `Send + Sync` are required by the Leptos context API, not by the runtime
/// (CSR is single-threaded).
pub type FeedbackSubmitFn = Arc<dyn Fn(&FeedbackReport) -> BoxedSubmitFuture + Send + Sync>;

/// Adapt a [`FeedbackSink`] into a storable [`FeedbackSubmitFn`].
pub fn sink_to_fn<S: FeedbackSink + Send + Sync + 'static>(sink: Arc<S>) -> FeedbackSubmitFn {
    Arc::new(move |report: &FeedbackReport| {
        let sink = Arc::clone(&sink);
        let report = report.clone();
        Box::pin(async move { sink.submit(&report).await })
    })
}

#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use crate::feedback::types::{
        FeedbackCategory, FeedbackEnvironment, FeedbackSource, FeedbackSubject,
    };

    fn report() -> FeedbackReport {
        FeedbackReport {
            category: FeedbackCategory::Translation,
            source: FeedbackSource::TranslatorPopup,
            subject: FeedbackSubject {
                surface: "食べる".to_string(),
                reading: Some("たべる".to_string()),
                context_line: None,
            },
            message: "test".to_string(),
            environment: FeedbackEnvironment {
                app_version: "test",
                platform: "web".to_string(),
                page: "/lesson".to_string(),
                ui_language: "en".to_string(),
            },
        }
    }

    /// A fake sink records submissions without touching `window.Sentry`.
    struct FakeSink;

    impl FeedbackSink for FakeSink {
        async fn submit(&self, _report: &FeedbackReport) -> Result<(), FeedbackSubmitError> {
            Ok(())
        }
    }

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn sink_to_fn_adapts_and_awaits() {
        let submit = sink_to_fn(Arc::new(FakeSink));
        let outcome = (submit)(&report()).await;
        assert!(outcome.is_ok());
    }
}
