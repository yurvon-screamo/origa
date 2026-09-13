//! Global feedback modal state (Leptos context).
//!
//! Entry points call [`FeedbackContext::open`] with the subject they captured;
//! the mounted [`crate::ui_components::FeedbackModal`] renders the form and
//! submits through the stored [`FeedbackSubmitFn`].

use std::sync::Arc;

use leptos::prelude::{RwSignal, Set};

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

/// Context provided once in `app.rs`. Holds the open draft and the transport.
#[derive(Clone)]
pub struct FeedbackContext {
    /// `Some(draft)` while the modal is open.
    pub draft: RwSignal<Option<FeedbackDraft>>,
    /// Type-erased submission transport (Sentry in production, fake in tests).
    pub submit: FeedbackSubmitFn,
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
        }
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
