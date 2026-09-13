//! WASM render tests for the feedback modal (`FeedbackModal`, issue #414).
//!
//! Run locally:
//! ```bash
//! wasm-pack test --headless --chrome origa_ui --features csr -- feedback_modal_wasm_tests
//! ```
//!
//! The modal is driven entirely by `FeedbackContext.draft` — tests provide
//! the context with a fake transport, open a draft, and assert the rendered
//! states. Connectivity is provided through the real `ConnectivityStore`
//! (online by default in a browser test).

#![cfg(all(target_arch = "wasm32", test))]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::feedback::sink::{FeedbackSink, sink_to_fn};
use crate::feedback::types::FeedbackEnvironment;
use crate::feedback::{
    FeedbackCategory, FeedbackContext, FeedbackReport, FeedbackSource, FeedbackSubject,
    FeedbackSubmitError,
};
use crate::store::connectivity::ConnectivityStore;
use crate::test_support::{create_wrapper, mount_to_wrapper};
use crate::ui_components::FeedbackModal;

wasm_bindgen_test_configure!(run_in_browser);

/// Fake transport: records submissions, returns the programmed outcome.
struct FakeSink {
    calls: Arc<AtomicUsize>,
    outcome: Result<(), FeedbackSubmitError>,
}

impl FeedbackSink for FakeSink {
    async fn submit(&self, _report: &FeedbackReport) -> Result<(), FeedbackSubmitError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.outcome.clone()
    }
}

struct Harness {
    ctx: FeedbackContext,
    calls: Arc<AtomicUsize>,
}

fn harness(outcome: Result<(), FeedbackSubmitError>) -> Harness {
    let calls = Arc::new(AtomicUsize::new(0));
    let sink = FakeSink {
        calls: Arc::clone(&calls),
        outcome,
    };
    Harness {
        ctx: FeedbackContext::with_submit(sink_to_fn(Arc::new(sink))),
        calls,
    }
}

fn draft() -> (FeedbackCategory, FeedbackSource, FeedbackSubject) {
    (
        FeedbackCategory::Translation,
        FeedbackSource::TranslatorPopup,
        FeedbackSubject {
            surface: "食べる".to_string(),
            reading: Some("たべる".to_string()),
            context_line: Some("毎日おいしいものを食べる".to_string()),
        },
    )
}

/// Mount the modal with the given context. Connectivity goes online; the
/// i18n context is provided the same way `mount_with_i18n` does (the modal
/// reads localized strings in setup).
fn mount_modal(ctx: &FeedbackContext) -> web_sys::Element {
    let wrapper = create_wrapper();
    let ctx = ctx.clone();
    mount_to_wrapper(&wrapper, move || {
        leptos_i18n::provide_i18n_context::<crate::i18n::Locale>();
        provide_context(ctx.clone());
        provide_context(ConnectivityStore::new());
        view! { <FeedbackModal /> }.into_any()
    });
    wrapper
}

async fn open_and_render(h: &Harness) -> web_sys::Element {
    let wrapper = mount_modal(&h.ctx);
    h.ctx.open(draft().0, draft().1, draft().2, "en");
    tick().await;
    wrapper
}

#[wasm_bindgen_test]
async fn opening_a_draft_shows_subject_and_form() {
    let h = harness(Ok(()));
    let wrapper = open_and_render(&h).await;

    let subject = wrapper
        .query_selector("[data-testid=\"feedback-subject\"]")
        .expect("query ok")
        .expect("subject rendered");
    let text = subject.text_content().unwrap_or_default();
    assert!(text.contains("食べる"), "subject surface visible: {text}");
    assert!(text.contains("たべる"), "subject reading visible: {text}");

    assert!(
        wrapper
            .query_selector("[data-testid=\"feedback-message-input\"]")
            .expect("query ok")
            .is_some(),
        "message input rendered"
    );
}

#[wasm_bindgen_test]
async fn submit_disabled_for_blank_message() {
    let h = harness(Ok(()));
    let wrapper = open_and_render(&h).await;

    let btn = wrapper
        .query_selector("[data-testid=\"feedback-submit-btn\"]")
        .expect("query ok")
        .expect("submit rendered");
    let disabled = btn.get_attribute("disabled");
    assert!(
        disabled.as_deref().is_some_and(|v| v != "false"),
        "blank message must disable submit (got {disabled:?})"
    );
}

#[wasm_bindgen_test]
async fn typing_text_and_submitting_shows_success() {
    let h = harness(Ok(()));
    let wrapper = open_and_render(&h).await;

    // Type into the bound textarea.
    let input = wrapper
        .query_selector("[data-testid=\"feedback-message-input\"]")
        .expect("query ok")
        .expect("input rendered");
    let textarea = input
        .clone()
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .unwrap();
    textarea.set_value("wrong gloss");

    // The bound signal updates through the input event; dispatch it.
    let event = web_sys::Event::new("input").unwrap();
    input.dispatch_event(&event).unwrap();
    tick().await;

    let btn = wrapper
        .query_selector("[data-testid=\"feedback-submit-btn\"]")
        .expect("query ok")
        .expect("submit rendered");
    assert!(
        btn.get_attribute("disabled").is_none(),
        "submit enabled for non-empty message"
    );

    btn.dyn_into::<web_sys::HtmlElement>().unwrap().click();
    tick().await;

    assert_eq!(h.calls.load(Ordering::SeqCst), 1, "one submission");

    let subject = wrapper.query_selector("[data-testid=\"feedback-subject\"]");
    assert!(
        subject.expect("query ok").is_none(),
        "success state replaces the form"
    );
}

#[wasm_bindgen_test]
async fn transport_failure_shows_error_with_retry() {
    let h = harness(Err(FeedbackSubmitError::SubmitFailed("boom".into())));
    let wrapper = open_and_render(&h).await;

    let input = wrapper
        .query_selector("[data-testid=\"feedback-message-input\"]")
        .expect("query ok")
        .expect("input rendered");
    let textarea = input
        .clone()
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .unwrap();
    textarea.set_value("message");
    let event = web_sys::Event::new("input").unwrap();
    input.dispatch_event(&event).unwrap();
    tick().await;

    wrapper
        .query_selector("[data-testid=\"feedback-submit-btn\"]")
        .expect("query ok")
        .expect("submit rendered")
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap()
        .click();
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"feedback-error\"]")
            .expect("query ok")
            .is_some(),
        "error state rendered"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"feedback-retry-btn\"]")
            .expect("query ok")
            .is_some(),
        "retry rendered after failure"
    );
    // Draft text preserved: the textarea still holds the typed message.
    let preserved = wrapper
        .query_selector("[data-testid=\"feedback-message-input\"]")
        .expect("query ok")
        .expect("input still rendered after error")
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .unwrap()
        .value();
    assert_eq!(preserved, "message", "draft preserved after error");
}

#[wasm_bindgen_test]
async fn compiled_out_dsn_shows_info_not_error() {
    let h = harness(Err(FeedbackSubmitError::SentryUnavailable));
    let wrapper = open_and_render(&h).await;

    let input = wrapper
        .query_selector("[data-testid=\"feedback-message-input\"]")
        .expect("query ok")
        .expect("input rendered");
    let textarea = input
        .clone()
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .unwrap();
    textarea.set_value("msg");
    let event = web_sys::Event::new("input").unwrap();
    input.dispatch_event(&event).unwrap();
    tick().await;

    wrapper
        .query_selector("[data-testid=\"feedback-submit-btn\"]")
        .expect("query ok")
        .expect("submit rendered")
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap()
        .click();
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"feedback-unavailable\"]")
            .expect("query ok")
            .is_some(),
        "info state rendered for compiled-out DSN"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"feedback-error\"]")
            .expect("query ok")
            .is_none(),
        "no error state for the informational case"
    );
}

#[wasm_bindgen_test]
async fn environment_snapshot_taken_on_open() {
    let h = harness(Ok(()));
    h.ctx.open(draft().0, draft().1, draft().2, "en");
    tick().await;

    let opened = h.ctx.draft.get_untracked().expect("draft set");
    assert_eq!(
        opened.environment.app_version,
        crate::core::version::VERSION
    );
    assert!(!opened.environment.platform.is_empty());
    assert!(!opened.environment.ui_language.is_empty());
}
