//! Global feedback modal state (Leptos context).
//!
//! Entry points call [`FeedbackContext::open`] with the subject they captured;
//! the mounted [`crate::ui_components::FeedbackModal`] renders the form and
//! submits through the stored [`FeedbackSubmitFn`].

use std::sync::Arc;

use leptos::prelude::{Get, RwSignal, Set};

use super::sink::{FeedbackSubmitFn, SentryFeedbackSink, sink_to_fn};
use super::types::{FeedbackCategory, FeedbackEnvironment, FeedbackSource, FeedbackSubject};

/// Everything the modal needs, assembled at `open()` time.
#[derive(Clone, Debug)]
pub struct FeedbackDraft {
    pub category: FeedbackCategory,
    pub source: FeedbackSource,
    pub subject: FeedbackSubject,
    /// Environment snapshot taken on open — matches what the user saw.
    pub environment: FeedbackEnvironment,
}

/// Anti-spam cooldown after a SUCCESSFUL submission (ADR-055): 15s. A
/// legitimate multi-report flow (several wrong tokens in one phrase) types
/// the next message for longer than that, so it is never blocked; an
/// immediate resubmit is.
pub const FEEDBACK_COOLDOWN_MS: f64 = 15_000.0;

/// Context provided once in `app.rs`. Holds the open draft and the transport.
#[derive(Clone)]
pub struct FeedbackContext {
    /// `Some(draft)` while the modal is open.
    pub draft: RwSignal<Option<FeedbackDraft>>,
    /// Type-erased submission transport (Sentry in production, fake in tests).
    pub submit: FeedbackSubmitFn,
    /// `Date::now()` ms of the last successful submission (cooldown source).
    last_success_at: RwSignal<Option<f64>>,
}

impl FeedbackContext {
    /// Create a production context (Sentry transport, modal closed).
    pub fn new() -> Self {
        Self::with_submit(sink_to_fn(Arc::new(SentryFeedbackSink)))
    }

    /// Create a context with an injected transport (wasm render tests).
    pub fn with_submit(submit: FeedbackSubmitFn) -> Self {
        Self {
            draft: RwSignal::new(None),
            submit,
            last_success_at: RwSignal::new(None),
        }
    }

    /// Whether an immediate resubmit is suppressed by the cooldown.
    pub fn is_cooling_down(&self) -> bool {
        self.last_success_at
            .get()
            .is_some_and(|sent_at| (js_sys::Date::now() - sent_at) < FEEDBACK_COOLDOWN_MS)
    }

    /// Record a successful submission timestamp (cooldown starts).
    pub fn note_submission(&self) {
        self.last_success_at.set(Some(js_sys::Date::now()));
    }

    /// Open the modal from an entry point. The environment is snapshotted
    /// here — at the moment of frustration, not at submit time.
    pub fn open(
        &self,
        category: FeedbackCategory,
        source: FeedbackSource,
        subject: FeedbackSubject,
        ui_language: &str,
    ) {
        self.draft.set(Some(FeedbackDraft {
            category,
            source,
            subject,
            environment: FeedbackEnvironment::capture(ui_language),
        }));
    }
}
