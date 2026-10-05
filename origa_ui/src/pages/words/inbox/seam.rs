//! e2e seam and route execution for the inbox.
//!
//! The seam lets Playwright deliver payloads without native share
//! mechanisms. It activates only when the test run opted in via localStorage
//! (see [`super::seam_enabled`]); in production the flag is never set, the
//! property is never registered and the intake stays native-only.

use super::{
    AcceptDecision, InboxKind, InboxPayload, InboxRoute, InboxSignals, ProcessContext,
    SEAM_TEXT_MAX_BYTES, SeamPayload, TranscribeContext, inbox_accept_policy, process_file,
    route_payload, seam_enabled, transcribe_file,
};
use crate::i18n::use_i18n;
use crate::pages::words::add_words_preview_modal_state::PreviewModalState;
use crate::ui_components::{ToastData, ToastType};
use leptos::prelude::*;
use tracing::{debug, warn};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;

pub(super) const SEAM_PROPERTY: &str = "__ORIGA_TEST_INBOX__";
const SEAM_FLAG_STORAGE_KEY: &str = "__origa_e2e_seam";

/// Owns the registered seam closure; dropping the guard removes the window
/// property so a disposed drawer cannot receive payloads.
pub(in crate::pages::words) struct InboxSeamGuard {
    window: web_sys::Window,
    _closure: Closure<dyn Fn(JsValue)>,
}

impl Drop for InboxSeamGuard {
    fn drop(&mut self) {
        let window_object = js_sys::Object::from(self.window.clone());
        let _ = js_sys::Reflect::delete_property(&window_object, &JsValue::from_str(SEAM_PROPERTY));
    }
}

fn read_seam_flag() -> Option<String> {
    let storage = web_sys::window()?.local_storage().ok().flatten()?;
    storage.get_item(SEAM_FLAG_STORAGE_KEY).ok().flatten()
}

/// Registers the seam when (and only when) the opt-in flag is present.
pub(in crate::pages::words) fn register_inbox_seam(
    state: PreviewModalState,
    is_open: RwSignal<bool>,
    inbox: InboxSignals,
    toasts: RwSignal<Vec<ToastData>>,
) -> Option<InboxSeamGuard> {
    if !seam_enabled(read_seam_flag()) {
        return None;
    }
    let window = web_sys::window()?;
    let i18n = use_i18n();

    let closure = Closure::wrap(Box::new(move |payload: JsValue| {
        handle_seam_payload(payload, &state, is_open, &inbox, toasts, i18n);
    }) as Box<dyn Fn(JsValue)>);

    let window_value: JsValue = window.clone().into();
    if js_sys::Reflect::set(
        &window_value,
        &JsValue::from_str(SEAM_PROPERTY),
        closure.as_ref(),
    )
    .is_err()
    {
        warn!("inbox seam: window property registration failed");
        return None;
    }
    debug!("inbox seam: registered");
    Some(InboxSeamGuard {
        window,
        _closure: closure,
    })
}

fn handle_seam_payload(
    payload: JsValue,
    state: &PreviewModalState,
    is_open: RwSignal<bool>,
    inbox: &InboxSignals,
    toasts: RwSignal<Vec<ToastData>>,
    i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
) {
    let parsed: SeamPayload = match serde_wasm_bindgen::from_value(payload) {
        Ok(parsed) => parsed,
        Err(e) => {
            warn!(error = ?e, "inbox seam: payload decode failed");
            return;
        },
    };

    let kind = match build_kind(parsed) {
        Some(kind) => kind,
        None => return,
    };

    let is_open_now = is_open.get_untracked();
    let is_processing_now = state.is_analyzing.get_untracked()
        || state.is_creating.get_untracked()
        || inbox.is_processing();
    let input_nonempty = is_open_now && !state.input_text.get_untracked().trim().is_empty();

    match inbox_accept_policy(is_open_now, is_processing_now, input_nonempty) {
        AcceptDecision::Accept => {
            execute_route(
                route_payload(InboxPayload { kind }),
                i18n,
                state,
                is_open,
                inbox,
            );
        },
        AcceptDecision::RejectBusy => {
            push_reject_toast(toasts, i18n, false);
        },
        AcceptDecision::RejectPendingInput => {
            push_reject_toast(toasts, i18n, true);
        },
    }
}

/// Builds the inbox kind from the wire payload. Malformed payloads are
/// logged and dropped — a test seam must never panic.
fn build_kind(parsed: SeamPayload) -> Option<InboxKind> {
    match parsed.kind.as_str() {
        "text" => {
            let text = parsed.text?;
            if text.len() > SEAM_TEXT_MAX_BYTES {
                warn!("inbox seam: text payload exceeds the cap, dropped");
                return None;
            }
            Some(InboxKind::Text(text))
        },
        "file" => {
            let file_name = parsed.file_name?;
            let mime = parsed.mime.unwrap_or_default();
            Some(InboxKind::File(construct_file(&file_name, &mime)))
        },
        other => {
            warn!(kind = other, "inbox seam: unknown payload kind");
            None
        },
    }
}

/// Builds a small placeholder-backed `File` in the page context. Byte
/// content is irrelevant for the classified routes: validation keys on the
/// name and MIME type, and the audio rejection fires before any read.
fn construct_file(file_name: &str, mime: &str) -> web_sys::File {
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::new_with_length(1024).into());
    let bag = web_sys::FilePropertyBag::new();
    web_sys::FilePropertyBag::set_type(&bag, mime);
    web_sys::File::new_with_str_sequence_and_options(&parts.into(), file_name, &bag)
        .expect("seam File construction must succeed in a browser context")
}

/// Applies the routed pipeline entry point. Every accepted route opens the
/// drawer; the zero-tap contract is that the user lands on processing or on
/// the word preview, never back on the source tabs.
pub(in crate::pages::words) fn execute_route(
    route: InboxRoute,
    i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    state: &PreviewModalState,
    is_open: RwSignal<bool>,
    inbox: &InboxSignals,
) {
    inbox.reset();
    is_open.set(true);

    match route {
        InboxRoute::Analyze(text) => {
            inbox.active.set(true);
            state.set_extracted_text(text);
        },
        InboxRoute::EmptyText => {
            // The drawer opens straight onto the manual input with the
            // no-words notice; the source tabs stay available because the
            // user is expected to retry by hand.
            inbox.empty_text.set(true);
        },
        InboxRoute::Unsupported { mime, name } => {
            inbox.active.set(true);
            inbox.error.set(Some(
                i18n.get_keys()
                    .words()
                    .inbox()
                    .unsupported_file()
                    .inner()
                    .to_string()
                    .replacen("{}", &format!("{name} ({mime})"), 1),
            ));
        },
        InboxRoute::OcrFile(file) => {
            inbox.active.set(true);
            let ctx = ProcessContext {
                image_preview: RwSignal::new(None),
                ocr_state: inbox.ocr_state,
                ocr_loading_state: inbox.ocr_loading_state,
                error_message: inbox.error,
                disposed: state.disposed,
            };
            let on_text_extracted = text_callback(state, *inbox);
            process_file(
                i18n,
                file,
                ctx,
                on_text_extracted,
                Callback::new(|_: String| {}),
            );
        },
        InboxRoute::SttFile(file) => {
            inbox.active.set(true);
            let ctx = TranscribeContext {
                audio_state: inbox.audio_state,
                status_text: inbox.audio_status_text,
                error_message: inbox.error,
                disposed: state.disposed,
            };
            let on_text_extracted = text_callback(state, *inbox);
            transcribe_file(
                i18n,
                file,
                ctx,
                on_text_extracted,
                Callback::new(|_: String| {}),
            );
        },
    }
}

/// The shared late-arrival fence for extraction results: once the inbox run
/// is cancelled or reset (`active` cleared), a result landing afterwards is
/// dropped instead of yanking the user back into the flow.
fn text_callback(state: &PreviewModalState, inbox: InboxSignals) -> Callback<String> {
    let state = state.clone();
    Callback::new(move |text: String| {
        if !inbox.active.get_untracked() {
            return;
        }
        state.set_extracted_text(text);
    })
}

fn push_reject_toast(
    toasts: RwSignal<Vec<ToastData>>,
    i18n: leptos_i18n::I18nContext<crate::i18n::Locale>,
    pending_input: bool,
) {
    debug!(pending_input, "inbox seam: payload rejected by policy");
    let title = i18n
        .get_keys_untracked()
        .common()
        .error()
        .inner()
        .to_string();
    let message = if pending_input {
        i18n.get_keys_untracked()
            .words()
            .inbox()
            .pending_input()
            .inner()
            .to_string()
    } else {
        i18n.get_keys_untracked()
            .words()
            .inbox()
            .busy()
            .inner()
            .to_string()
    };
    toasts.update(|list| {
        list.push(ToastData {
            id: list.len(),
            toast_type: ToastType::Info,
            title,
            message,
            duration_ms: Some(4000),
            closable: true,
        });
    });
}
