//! In-app user feedback (issue #414, plan: Sentry `captureFeedback` channel).
//!
//! Architecture (ADR-055):
//!
//! - [`types`] — the report model: category/source (from the entry point),
//!   subject (the content the user is looking at), message, environment
//!   snapshot taken on open.
//! - [`sink`] — transport abstraction + Sentry implementation.
//! - [`context`] — global modal state; entry points call `open(...)`.
//!
//! Entry points live next to the frustration moment: the translator token
//! popup and the lesson header button. They are deliberately *not* generic
//! "feedback" buttons — the value is the captured subject.

pub mod context;
pub mod sink;
pub mod types;

pub use context::FeedbackContext;
pub use sink::FeedbackSubmitError;
pub use types::{
    FEEDBACK_MESSAGE_MAX_CHARS, FeedbackCategory, FeedbackReport, FeedbackSource, FeedbackSubject,
};

/// Current router path for the report environment (e.g. `/lesson`).
///
/// Reads `window.location.pathname` directly: in CSR the router path mirrors
/// the document location, and the DOM read cannot panic outside a `<Router>`
/// (component-test mounts) — returns `"?"` there instead.
pub fn current_page_path() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| "?".to_string())
}
