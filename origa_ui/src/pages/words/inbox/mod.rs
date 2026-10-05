//! Inbox: zero-tap content intake for the add-words flow.
//!
//! A payload arriving from outside the app (share sheet, open-with, iOS
//! share extension — wired in later slices) must land the user directly on
//! the word preview, skipping the source tabs. This module owns the pure
//! decision layer — content classification, routing and the accept policy —
//! so every branch is unit-testable without a DOM.
//!
//! Scope note (slice IN-1): payloads can only arrive while the add-words
//! drawer is mounted, i.e. the user is on `/words` and authenticated.
//! Parking payloads before authentication is the responsibility of the
//! native adapters (IN-2/3/4).

use crate::ui_components::OcrLoadingState;
use leptos::prelude::*;

#[cfg(all(target_arch = "wasm32", test))]
mod inbox_wasm_tests;
pub(super) mod seam;
pub(super) mod view;

// Sibling pipelines re-exported so seam/view use a single import root.
pub(super) use super::audio_transcribe::{
    AudioState, StaleRun, TranscribeContext, transcribe_file,
};
pub(super) use super::image_input_stage::stage_item_view;
pub(super) use super::ocr_processing::{OcrState, ProcessContext, process_file};
pub(super) use seam::{InboxSeamGuard, register_inbox_seam};
pub(super) use view::InboxStageView;

/// Reactive handles the inbox shares with the drawer view. Copy — every
/// field is a signal or a Copy state struct, so handlers can capture it
/// freely.
#[derive(Clone, Copy)]
pub(super) struct InboxSignals {
    /// True while a zero-tap run owns the drawer (source tabs hidden).
    pub active: RwSignal<bool>,
    /// Extraction failure message (OCR/STT/unsupported), rendered by the
    /// inbox failure view.
    pub error: RwSignal<Option<String>>,
    /// Drives the no-words notice for an empty-text payload.
    pub empty_text: RwSignal<bool>,
    pub ocr_state: RwSignal<super::ocr_processing::OcrState>,
    pub ocr_loading_state: OcrLoadingState,
    pub audio_state: RwSignal<super::audio_transcribe::AudioState>,
    pub audio_status_text: RwSignal<Option<String>>,
    /// Monotonic run counter. Each accepted route takes a run id; a result
    /// landing after any bump (new payload or cancel) belongs to a stale run
    /// and must be dropped instead of feeding the current one.
    pub generation: RwSignal<u32>,
}

impl InboxSignals {
    pub fn new() -> Self {
        Self {
            active: RwSignal::new(false),
            error: RwSignal::new(None),
            empty_text: RwSignal::new(false),
            ocr_state: RwSignal::new(super::ocr_processing::OcrState::Idle),
            ocr_loading_state: OcrLoadingState::new(),
            audio_state: RwSignal::new(super::audio_transcribe::AudioState::Idle),
            audio_status_text: RwSignal::new(None),
            generation: RwSignal::new(0),
        }
    }

    /// Opens a new run: invalidates every previous one and returns its id.
    pub fn next_run(&self) -> u32 {
        self.generation.update(|g| *g = g.wrapping_add(1));
        self.generation.get_untracked()
    }

    /// True while any extraction pipeline is mid-flight. Drives the accept
    /// policy: a second payload must not race the in-flight one.
    pub fn is_processing(&self) -> bool {
        use super::audio_transcribe::AudioState;
        use super::ocr_processing::OcrState;

        self.ocr_state.get_untracked() == OcrState::Processing
            || matches!(
                self.audio_state.get_untracked(),
                AudioState::LoadingModel | AudioState::Processing
            )
    }

    /// Clears the inbox back to the pre-payload state and invalidates any
    /// in-flight run so its late result cannot touch the fresh state.
    pub fn reset(&self) {
        use super::audio_transcribe::AudioState;
        use super::ocr_processing::OcrState;

        self.generation.update(|g| *g = g.wrapping_add(1));
        self.active.set(false);
        self.error.set(None);
        self.empty_text.set(false);
        self.ocr_state.set(OcrState::Idle);
        self.ocr_loading_state.reset();
        self.audio_state.set(AudioState::Idle);
        self.audio_status_text.set(None);
    }
}

impl Default for InboxSignals {
    fn default() -> Self {
        Self::new()
    }
}

/// A unit of externally supplied content. Files are constructed by the
/// native adapters (or the e2e seam) as `web_sys::File`; exact format and
/// size validation stays in the shared pipelines (`process_file` /
/// `transcribe_file`), the inbox only classifies.
#[derive(Clone)]
pub(super) enum InboxKind {
    Text(String),
    File(web_sys::File),
}

pub(super) struct InboxPayload {
    pub kind: InboxKind,
}

/// Coarse content class a payload routes to. Text exists only for direct
/// string payloads (share text); text files are out of scope for IN-1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FileClass {
    Image,
    Audio,
    Unsupported,
}

/// Where a payload goes once accepted.
#[derive(Debug)]
pub(super) enum InboxRoute {
    Analyze(String),
    OcrFile(web_sys::File),
    SttFile(web_sys::File),
    Unsupported { mime: String, name: String },
    EmptyText,
}

/// Result of the accept policy for an incoming payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AcceptDecision {
    Accept,
    /// A pipeline is already running for this drawer — the payload would
    /// race the in-flight one.
    RejectBusy,
    /// The drawer holds non-empty user input that would be silently
    /// destroyed by the payload.
    RejectPendingInput,
}

const TEXT_EXTENSIONS: &[&str] = &[".txt"];
const IMAGE_EXTENSIONS: &[&str] = &[".png", ".jpg", ".jpeg", ".webp"];
const AUDIO_EXTENSIONS: &[&str] = &[".wav", ".mp3", ".webm", ".m4a", ".ogg"];

/// Classifies a file payload by MIME prefix, falling back to the extension
/// when the share path supplies no type. Only the coarse class decides the
/// route; per-format rules (e.g. WAV-only on the WASM fallback, the 10 MB
/// OCR cap) are enforced by the shared pipelines, not here.
pub(super) fn classify_file(mime: &str, file_name: &str) -> FileClass {
    let mime = mime.trim().to_ascii_lowercase();
    if mime.starts_with("image/") {
        return FileClass::Image;
    }
    if mime.starts_with("audio/") {
        return FileClass::Audio;
    }

    let lower_name = file_name.to_ascii_lowercase();
    let has_extension = |extensions: &[&str]| {
        extensions
            .iter()
            .any(|extension| lower_name.ends_with(extension))
    };

    if has_extension(IMAGE_EXTENSIONS) {
        FileClass::Image
    } else if has_extension(AUDIO_EXTENSIONS) {
        FileClass::Audio
    } else if has_extension(TEXT_EXTENSIONS) {
        // Text files require reading content into a string — a later slice;
        // for now they surface as unsupported instead of a dead end.
        FileClass::Unsupported
    } else {
        FileClass::Unsupported
    }
}

/// Routes an accepted payload to its pipeline entry point.
///
/// Whitespace-only text routes to [`InboxRoute::EmptyText`] instead of
/// running an analysis that is guaranteed to find nothing.
pub(super) fn route_payload(payload: InboxPayload) -> InboxRoute {
    match payload.kind {
        InboxKind::Text(text) => {
            if text.trim().is_empty() {
                InboxRoute::EmptyText
            } else {
                InboxRoute::Analyze(text)
            }
        },
        InboxKind::File(file) => {
            let mime = file.type_();
            let name = file.name();
            match classify_file(&mime, &name) {
                FileClass::Image => InboxRoute::OcrFile(file),
                FileClass::Audio => InboxRoute::SttFile(file),
                FileClass::Unsupported => InboxRoute::Unsupported { mime, name },
            }
        },
    }
}

/// Decides whether the inbox may take a new payload right now.
///
/// A closed drawer always accepts (reset clears any stale input first).
/// An open drawer accepts only when nothing is running and the user has not
/// typed anything that would be silently destroyed.
pub(super) fn inbox_accept_policy(
    is_open: bool,
    is_processing: bool,
    user_input_nonempty: bool,
) -> AcceptDecision {
    if !is_open {
        AcceptDecision::Accept
    } else if is_processing {
        AcceptDecision::RejectBusy
    } else if user_input_nonempty {
        AcceptDecision::RejectPendingInput
    } else {
        AcceptDecision::Accept
    }
}

/// The e2e seam gate. The seam exists so Playwright can deliver payloads
/// without native share mechanisms; it activates only when the test run has
/// opted in via localStorage, identically in debug and release builds (the
/// CI e2e dist is the same artifact reused by the production bundles, so a
/// compile-time flag would leak the seam into production).
pub(super) fn seam_enabled(stored_flag: Option<String>) -> bool {
    stored_flag.as_deref() == Some("1")
}

/// Wire-format of the seam payload. `kind` is `"text"` or `"file"`.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SeamPayload {
    pub kind: String,
    pub text: Option<String>,
    pub file_name: Option<String>,
    pub mime: Option<String>,
}

/// Upper bound for seam text payloads — keeps an accidental huge string in
/// a test session from ballooning the WASM heap. File payloads are bounded
/// by the shared pipelines' own caps.
pub(super) const SEAM_TEXT_MAX_BYTES: usize = 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case::png_mime("image/png", "photo", FileClass::Image)]
    #[case::jpeg_mime("image/jpeg", "photo.bin", FileClass::Image)]
    #[case::audio_mime("audio/mpeg", "episode.bin", FileClass::Audio)]
    #[case::extension_fallback_png("", "homework.PNG", FileClass::Image)]
    #[case::extension_fallback_mp3("  ", "podcast.mp3", FileClass::Audio)]
    #[case::text_file_is_unsupported_for_now(
        "application/octet-stream",
        "notes.txt",
        FileClass::Unsupported
    )]
    #[case::unknown_binary("application/zip", "archive.zip", FileClass::Unsupported)]
    #[case::no_mime_no_extension("", "file", FileClass::Unsupported)]
    fn file_classification_covers_mime_prefix_then_extension(
        #[case] mime: &str,
        #[case] file_name: &str,
        #[case] expected: FileClass,
    ) {
        assert_eq!(classify_file(mime, file_name), expected);
    }

    #[rstest]
    #[case::closed_drawer_accepts_even_with_input(false, true, true, AcceptDecision::Accept)]
    #[case::closed_drawer_accepts_idle(false, false, false, AcceptDecision::Accept)]
    #[case::open_idle_accepts(true, false, false, AcceptDecision::Accept)]
    #[case::processing_rejects(true, true, false, AcceptDecision::RejectBusy)]
    #[case::processing_beats_pending_input(true, true, true, AcceptDecision::RejectBusy)]
    #[case::pending_input_rejects(true, false, true, AcceptDecision::RejectPendingInput)]
    fn accept_policy_protects_in_flight_work_and_user_input(
        #[case] is_open: bool,
        #[case] is_processing: bool,
        #[case] user_input_nonempty: bool,
        #[case] expected: AcceptDecision,
    ) {
        assert_eq!(
            inbox_accept_policy(is_open, is_processing, user_input_nonempty),
            expected
        );
    }

    #[rstest]
    #[case::exact_opt_in_value(Some("1".to_string()), true)]
    #[case::wrong_value(Some("0".to_string()), false)]
    #[case::key_absent(None, false)]
    fn seam_gate_requires_exact_opt_in_flag(
        #[case] stored_flag: Option<String>,
        #[case] expected: bool,
    ) {
        assert_eq!(seam_enabled(stored_flag), expected);
    }

    #[test]
    fn text_payload_routes_by_whitespace_only() {
        let meaningful = route_payload(InboxPayload {
            kind: InboxKind::Text("私は本を読みます".to_string()),
        });
        assert!(matches!(meaningful, InboxRoute::Analyze(_)));

        let blank = route_payload(InboxPayload {
            kind: InboxKind::Text("   \n\t".to_string()),
        });
        assert!(matches!(blank, InboxRoute::EmptyText));
    }

    #[test]
    fn seam_payload_deserializes_from_camel_case() {
        let raw = r#"{"kind":"file","fileName":"a.mp3","mime":"audio/mpeg"}"#;
        let payload: SeamPayload = serde_json::from_str(raw).expect("seam payload must parse");
        assert_eq!(payload.file_name.as_deref(), Some("a.mp3"));
        assert_eq!(payload.mime.as_deref(), Some("audio/mpeg"));
    }
}
