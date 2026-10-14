//! Onboarding step telemetry: `<step>_start` / `<step>_complete` Umami
//! events for the step-wise funnel (issue #774, ADR-054 §5 programmatic
//! slice).
//!
//! Event names follow the step enum identity (`onboarding_intro_start` …
//! `onboarding_scoring_start`), not the localized stepper labels — the
//! slug↔label table lives in ADR-054. Scoring completion is deliberately
//! NOT emitted here: the declarative `onboarding_complete` on the scoring
//! finish button already is that event (kept for continuity of the existing
//! login_submit → onboarding_complete funnel). Skip paths stay silent, so
//! they remain visible as funnel drop-off (ADR-054 §5 semantics).

use super::onboarding_state::OnboardingStep;
use crate::core::analytics;

/// Which boundary of a step the event marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StepPhase {
    Start,
    Complete,
}

/// Lowercase enum slug — the single source for event names. `load` is the
/// daily-load pace step and `summary` is the import step (stepper labels
/// "pace"/"import"); the mapping to user-facing labels is tabulated in
/// ADR-054 §5.
fn step_slug(step: OnboardingStep) -> &'static str {
    match step {
        OnboardingStep::Intro => "intro",
        OnboardingStep::Load => "load",
        OnboardingStep::Jlpt => "jlpt",
        OnboardingStep::Apps => "apps",
        OnboardingStep::Progress => "progress",
        OnboardingStep::Summary => "summary",
        OnboardingStep::Scoring => "scoring",
    }
}

fn phase_suffix(phase: StepPhase) -> &'static str {
    match phase {
        StepPhase::Start => "start",
        StepPhase::Complete => "complete",
    }
}

/// `onboarding_<slug>_<start|complete>`.
fn event_name(step: OnboardingStep, phase: StepPhase) -> String {
    format!("onboarding_{}_{}", step_slug(step), phase_suffix(phase))
}

/// Pure device classifier for event properties. Mobile when the UA
/// identifies a phone/tablet browser, or when a Macintosh UA carries real
/// touch — the canonical iPadOS 13+ desktop-mode signature (`navigator.
/// platform` reports "MacIntel" there, but `maxTouchPoints` is >1 on iPads
/// and 0–1 on desktop Macs). Note: Umami's own device dimension is UA-only
/// and keeps classifying those iPads as desktop — this property
/// deliberately diverges (see ADR-054 §5).
fn classify_device(user_agent: &str, max_touch_points: u32) -> &'static str {
    let ua = user_agent.to_lowercase();
    let phone_class = ["android", "iphone", "ipad", "ipod", "mobile"]
        .iter()
        .any(|token| ua.contains(token));
    let ipad_desktop_mode = ua.contains("macintosh") && max_touch_points > 1;
    if phone_class || ipad_desktop_mode {
        "mobile"
    } else {
        "desktop"
    }
}

/// Live device read for event properties. Non-DOM falls back to "desktop":
/// telemetry is best-effort and must never panic (ADR-054 contract).
/// `pub(super)` so the scoring finish button can attach the same property
/// to its declarative `onboarding_complete` event.
pub(super) fn device() -> &'static str {
    let Some(window) = web_sys::window() else {
        return "desktop";
    };
    let navigator = window.navigator();
    let user_agent = navigator.user_agent().unwrap_or_default();
    // web-sys exposes the IDL `long` — never negative in practice, so a
    // failed conversion (defensive) degrades to "no touch".
    let max_touch_points = u32::try_from(navigator.max_touch_points()).unwrap_or(0);
    classify_device(&user_agent, max_touch_points)
}

/// Emits `onboarding_<step>_<phase>` with `locale` + `device` event
/// properties. Fire-and-forget: telemetry failures are logged inside the
/// analytics layer, never surfaced to the flow.
fn track_step(step: OnboardingStep, phase: StepPhase, locale: &str) {
    analytics::track_event(
        &event_name(step, phase),
        &[("locale", locale), ("device", device())],
    );
}

/// sessionStorage key of the last started step slug in this tab session.
const LAST_STARTED_KEY: &str = "umami_last_start_step";

/// Pure dedup decision: `true` when `step` differs from the recorded slug.
fn is_start_duplicate(last: Option<String>, step: OnboardingStep) -> bool {
    last.as_deref() == Some(step_slug(step))
}

fn read_last_started() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.session_storage().ok()??;
    storage.get_item(LAST_STARTED_KEY).ok()?
}

fn write_last_started(slug: &str) -> Option<()> {
    let window = web_sys::window()?;
    let storage = window.session_storage().ok()??;
    storage.set_item(LAST_STARTED_KEY, slug).ok()
}

/// Dedup decision for start events: `true` only when `step` differs from
/// the last started one (mutates the dedup state). Storage failures degrade
/// to emitting — attempt semantics beat missing data.
fn should_emit_start(step: OnboardingStep) -> bool {
    let last = read_last_started();
    if is_start_duplicate(last, step) {
        return false;
    }
    let _ = write_last_started(step_slug(step));
    true
}

/// Step entry: `onboarding_<step>_start`, deduped against remounts and
/// in-tab reloads.
pub(super) fn track_step_start(step: OnboardingStep, locale: &str) {
    if should_emit_start(step) {
        track_step(step, StepPhase::Start, locale);
    }
}

/// Step forward action succeeded: `onboarding_<step>_complete` (fires once
/// per transition point — no dedup needed).
pub(super) fn track_step_complete(step: OnboardingStep, locale: &str) {
    track_step(step, StepPhase::Complete, locale);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(OnboardingStep::Intro, StepPhase::Start, "onboarding_intro_start")]
    #[case(
        OnboardingStep::Intro,
        StepPhase::Complete,
        "onboarding_intro_complete"
    )]
    #[case(OnboardingStep::Load, StepPhase::Start, "onboarding_load_start")]
    #[case(OnboardingStep::Load, StepPhase::Complete, "onboarding_load_complete")]
    #[case(OnboardingStep::Jlpt, StepPhase::Start, "onboarding_jlpt_start")]
    #[case(OnboardingStep::Jlpt, StepPhase::Complete, "onboarding_jlpt_complete")]
    #[case(OnboardingStep::Apps, StepPhase::Start, "onboarding_apps_start")]
    #[case(OnboardingStep::Apps, StepPhase::Complete, "onboarding_apps_complete")]
    #[case(
        OnboardingStep::Progress,
        StepPhase::Start,
        "onboarding_progress_start"
    )]
    #[case(
        OnboardingStep::Progress,
        StepPhase::Complete,
        "onboarding_progress_complete"
    )]
    #[case(OnboardingStep::Summary, StepPhase::Start, "onboarding_summary_start")]
    #[case(
        OnboardingStep::Summary,
        StepPhase::Complete,
        "onboarding_summary_complete"
    )]
    #[case(OnboardingStep::Scoring, StepPhase::Start, "onboarding_scoring_start")]
    fn event_name_follows_step_enum_identity(
        #[case] step: OnboardingStep,
        #[case] phase: StepPhase,
        #[case] expected: &str,
    ) {
        assert_eq!(event_name(step, phase), expected);
    }

    #[rstest]
    // Android phone
    #[case(
        "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Mobile Safari/537.36",
        0,
        "mobile"
    )]
    // iPhone
    #[case(
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
        0,
        "mobile"
    )]
    // iPad in mobile mode (iPad UA token)
    #[case(
        "Mozilla/5.0 (iPad; CPU OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1",
        5,
        "mobile"
    )]
    // iPad in desktop mode (default since iPadOS 13): Mac UA + real touch
    #[case(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Safari/605.1.15",
        5,
        "mobile"
    )]
    // Desktop Mac (no touch)
    #[case(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        0,
        "desktop"
    )]
    // Windows desktop
    #[case(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        0,
        "desktop"
    )]
    // Touch-capable Windows laptop: touch alone must not mean mobile
    #[case(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        10,
        "desktop"
    )]
    // Linux desktop with an accidental single touch point
    #[case(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        1,
        "desktop"
    )]
    fn classify_device_matrix(
        #[case] user_agent: &str,
        #[case] max_touch_points: u32,
        #[case] expected: &str,
    ) {
        assert_eq!(classify_device(user_agent, max_touch_points), expected);
    }

    #[test]
    fn start_dedup_decisions() {
        // Fresh session (nothing recorded): the first entry emits.
        assert!(!is_start_duplicate(None, OnboardingStep::Intro));
        // A sync-overlay remount or an in-tab reload re-runs the effect with
        // the same step — suppressed via the recorded slug.
        assert!(is_start_duplicate(
            Some("intro".to_string()),
            OnboardingStep::Intro
        ));
        // Forward navigation emits the new step.
        assert!(!is_start_duplicate(
            Some("intro".to_string()),
            OnboardingStep::Load
        ));
        // Back-navigation re-entry emits: the step differs from the last
        // started one.
        assert!(!is_start_duplicate(
            Some("load".to_string()),
            OnboardingStep::Intro
        ));
        // Resume jump across steps emits.
        assert!(!is_start_duplicate(
            Some("intro".to_string()),
            OnboardingStep::Scoring
        ));
    }
}
