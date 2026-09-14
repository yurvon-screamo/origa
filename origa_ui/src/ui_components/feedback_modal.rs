//! Feedback modal (issue #414, ADR-055): the single form every entry point
//! opens. Shows the captured subject, collects the user's description, and
//! submits through the [`FeedbackContext`] transport.
//!
//! States: `Idle` → `Sending` (closing blocked) → `Success` (auto-close) |
//! `Error` (retry, draft preserved) | `Unavailable` (info: compiled-out DSN).

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::feedback::{
    FEEDBACK_MESSAGE_MAX_CHARS, FeedbackContext, FeedbackReport, FeedbackSubmitError,
};
use crate::i18n::use_i18n;
use crate::store::connectivity::ConnectivityStore;

use super::{Button, ButtonVariant, Input, Modal};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FormState {
    Idle,
    Sending,
    Success,
    Error,
    /// DSN compiled out (dev/e2e build) — informational, not an error.
    Unavailable,
}

/// Global feedback modal. Mount once (app shell) next to `ToastContainer`.
/// Opening is driven by `FeedbackContext.draft`, not by props.
#[component]
pub fn FeedbackModal() -> impl IntoView {
    let i18n = use_i18n();
    let feedback = use_context::<FeedbackContext>().expect("FeedbackContext not provided");
    let connectivity = use_context::<ConnectivityStore>();

    let draft = feedback.draft;
    let is_open = RwSignal::new(false);
    let message = RwSignal::new(String::new());
    let state = RwSignal::new(FormState::Idle);
    let show_auto_context = RwSignal::new(false);

    // Opening a draft (re)initializes the form: fresh textarea, fresh state.
    // A resubmission after Error keeps the draft but resets Sending.
    Effect::new(move |_| {
        if draft.get().is_some() {
            message.set(String::new());
            state.set(FormState::Idle);
            show_auto_context.set(false);
            is_open.set(true);
        }
    });

    // Reset the stored error/retry state after the modal fully closed so the
    // next open starts clean (message itself is reset by the open effect).
    Effect::new(move |_| {
        if !is_open.get() && draft.get().is_none() {
            state.set(FormState::Idle);
        }
    });

    let close = Callback::new(move |_: ()| {
        is_open.set(false);
        draft.set(None);
    });

    let message_trimmed_empty = Signal::derive(move || message.get().trim().is_empty());
    let is_offline =
        Signal::derive(move || connectivity.as_ref().is_some_and(|c| !c.is_online.get()));
    let feedback_for_gating = feedback.clone();
    let can_submit = Signal::derive(move || {
        state.get() == FormState::Idle
            && !message_trimmed_empty.get()
            && !is_offline.get()
            && !feedback_for_gating.is_cooling_down()
    });

    let submit = Callback::new(move |_: ()| {
        let Some(current_draft) = draft.get_untracked() else {
            return;
        };
        let text = message.get_untracked().trim().to_string();
        if text.is_empty() {
            return;
        }

        state.set(FormState::Sending);
        let submit_fn = feedback.submit.clone();
        let feedback_for_cooldown = feedback.clone();
        let report = FeedbackReport {
            category: current_draft.category,
            source: current_draft.source,
            subject: current_draft.subject.clone(),
            message: text,
            environment: current_draft.environment.clone(),
        };

        spawn_local(async move {
            match (submit_fn)(&report).await {
                Ok(()) => {
                    feedback_for_cooldown.note_submission();
                    state.set(FormState::Success);
                    // Auto-close after the check-draw animation had its time.
                    gloo_timers::future::TimeoutFuture::new(1500).await;
                    if state.get_untracked() == FormState::Success {
                        close.run(());
                    }
                },
                Err(FeedbackSubmitError::SentryUnavailable) => {
                    state.set(FormState::Unavailable);
                },
                Err(FeedbackSubmitError::SubmitFailed(reason)) => {
                    tracing::error!(error = %reason, "Feedback submission failed");
                    state.set(FormState::Error);
                },
            }
        });
    });

    let auto_context_line = Signal::derive(move || {
        draft
            .get()
            .map(|d| {
                format!(
                    "{} · {} · {} · {}",
                    d.environment.app_version,
                    d.environment.platform,
                    d.environment.page,
                    d.environment.ui_language,
                )
            })
            .unwrap_or_default()
    });

    let block_close = Signal::derive(move || state.get() == FormState::Sending);

    let title = Signal::derive(move || i18n.get_keys().feedback().title().inner().to_string());

    view! {
        <Modal
            test_id=Signal::derive(|| "feedback-modal".to_string())
            is_open=is_open
            title=title
            block_close=block_close
        >
            <Show
                when=move || state.get() == FormState::Success
                fallback=move || {
                    view! {
                        <div class="feedback-form" data-testid="feedback-form">
                            // Subject block: the content the user is reporting.
                            <Show when=move || draft.get().is_some_and(|d| !d.subject.surface.is_empty())>
                                <div class="feedback-subject" data-testid="feedback-subject">
                                    <div class="feedback-subject__surface font-serif">
                                        {move || draft.get().map(|d| d.subject.surface.clone()).unwrap_or_default()}
                                    </div>
                                    <Show when=move || draft.get().is_some_and(|d| d.subject.reading.as_deref().is_some_and(|r| !r.is_empty()))>
                                        <div class="feedback-subject__reading">
                                            {move || draft.get().and_then(|d| d.subject.reading).unwrap_or_default()}
                                        </div>
                                    </Show>
                                    <Show when=move || draft.get().is_some_and(|d| d.subject.context_line.as_deref().is_some_and(|l| !l.is_empty()))>
                                        <div class="feedback-subject__context">
                                            {move || draft.get().and_then(|d| d.subject.context_line).unwrap_or_default()}
                                        </div>
                                    </Show>
                                </div>
                            </Show>

                            // Message input.
                            <label class="feedback-label" for="feedback-message">
                                {move || i18n.get_keys().feedback().message_label().inner().to_string() }
                                <span class="feedback-label__required" aria-hidden="true">"*"</span>
                            </label>
                            <Input
                                value=message
                                rows=Signal::derive(|| Some(4usize))
                                maxlength=Signal::derive(|| Some(FEEDBACK_MESSAGE_MAX_CHARS))
                                placeholder=Signal::derive(move || {
                                    i18n.get_keys().feedback().message_placeholder().inner().to_string()
                                })
                                disabled=Signal::derive(move || state.get() == FormState::Sending)
                                id=Signal::derive(|| "feedback-message".to_string())
                                test_id=Signal::derive(|| "feedback-message-input".to_string())
                            />

                            // Auto-context disclosure.
                            <button
                                class="feedback-auto-toggle"
                                data-testid="feedback-auto-toggle"
                                aria-expanded=move || show_auto_context.get().to_string()
                                on:click=move |_| show_auto_context.update(|v| *v = !*v)
                            >
                                {move || i18n.get_keys().feedback().auto_context().inner().to_string()}
                                <span class="feedback-auto-toggle__caret">
                                    {move || if show_auto_context.get() { "−" } else { "+" }}
                                </span>
                            </button>
                            <Show when=move || show_auto_context.get()>
                                <div class="feedback-auto-line" data-testid="feedback-auto-context">
                                    {auto_context_line}
                                </div>
                            </Show>

                            <Show when=move || is_offline.get()>
                                <div class="feedback-offline-hint" data-testid="feedback-offline-hint">
                                    {move || i18n.get_keys().feedback().offline_hint().inner().to_string()}
                                </div>
                            </Show>

                            <Show when=move || state.get() == FormState::Unavailable>
                                <div class="feedback-info" data-testid="feedback-unavailable" role="status">
                                    {move || i18n.get_keys().feedback().release_only().inner().to_string()}
                                </div>
                            </Show>
                            <Show when=move || state.get() == FormState::Error>
                                <div class="feedback-error" data-testid="feedback-error" role="alert">
                                    {move || i18n.get_keys().feedback().submit_failed().inner().to_string()}
                                </div>
                            </Show>

                            <div class="feedback-privacy">
                                {move || i18n.get_keys().feedback().privacy_note().inner().to_string()}
                            </div>

                            <div class="feedback-actions">
                                <Button
                                    variant=Signal::derive(|| ButtonVariant::Default)
                                    on_click=Callback::new(move |ev: leptos::ev::MouseEvent| {
                                        ev.stop_propagation();
                                        close.run(());
                                    })
                                    disabled=block_close
                                    test_id=Signal::derive(|| "feedback-cancel-btn".to_string())
                                >
                                    {move || i18n.get_keys().feedback().cancel().inner().to_string()}
                                </Button>
                                <Show
                                    when=move || state.get() == FormState::Error
                                    fallback=move || view! {
                                        <Button
                                            variant=Signal::derive(|| ButtonVariant::Filled)
                                            on_click=Callback::new(move |ev: leptos::ev::MouseEvent| {
                                                ev.stop_propagation();
                                                submit.run(());
                                            })
                                            disabled=Signal::derive(move || !can_submit.get())
                                            loading=Signal::derive(move || state.get() == FormState::Sending)
                                            test_id=Signal::derive(|| "feedback-submit-btn".to_string())
                                        >
                                            {move || i18n.get_keys().feedback().submit().inner().to_string()}
                                        </Button>
                                    }
                                >
                                    <Button
                                        variant=Signal::derive(|| ButtonVariant::Filled)
                                        on_click=Callback::new(move |ev: leptos::ev::MouseEvent| {
                                            ev.stop_propagation();
                                            submit.run(());
                                        })
                                        disabled=Signal::derive(move || message_trimmed_empty.get() || is_offline.get())
                                        test_id=Signal::derive(|| "feedback-retry-btn".to_string())
                                    >
                                        {move || i18n.get_keys().feedback().retry().inner().to_string()}
                                    </Button>
                                </Show>
                            </div>
                        </div>
                    }
                }
            >
                <div class="feedback-success" data-testid="feedback-success">
                    <svg
                        class="feedback-success__check"
                        width="48"
                        height="48"
                        viewBox="0 0 48 48"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        aria-hidden="true"
                    >
                        <path d="M10 25l10 10 18-20" />
                    </svg>
                    <div class="feedback-success__title font-serif">
                        {move || i18n.get_keys().feedback().sent().inner().to_string()}
                    </div>
                    <div class="feedback-success__note">
                        {move || i18n.get_keys().feedback().sent_note().inner().to_string()}
                    </div>
                </div>
            </Show>
        </Modal>
    }
}
