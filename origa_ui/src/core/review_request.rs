//! In-app store review prompt gate (issue #729, ADR-062).
//!
//! After each completed study session the lesson complete screen calls
//! [`maybe_request_app_review`]. The gate keeps the cadence deliberately
//! slow: Apple caps `SKStoreReviewController` at 3 prompts per 365 days,
//! and the 120-day cooldown matches that ceiling, so no request is ever
//! wasted on a system-imposed rejection.

use gloo_storage::{LocalStorage, Storage};
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsValue;

use super::tauri::{invoke_with_args, is_tauri};

// pub(crate) — the wasm test module asserts on these keys directly.
pub(crate) const LAST_PROMPT_KEY: &str = "origa_review_request_last_prompt_ts";
pub(crate) const SESSIONS_KEY: &str = "origa_review_request_completed_sessions";

/// Apple silently drops prompts beyond 3 per 365 days; 120 days keeps the
/// app at that ceiling (3 requests/year) without ever exceeding it.
const COOLDOWN_SECS: u64 = 120 * 24 * 60 * 60;

/// A brand-new user must finish a few sessions before the first prompt —
/// an early prompt to a confused beginner risks a low rating, and the
/// program exists to build a pool of positive ratings (issue #729).
const MIN_SESSIONS_BEFORE_FIRST_PROMPT: u64 = 3;

/// Pure decision core: a prompt is due when the user has finished enough
/// sessions and the previous prompt (if any) is outside the cooldown.
///
/// `now - last >= COOLDOWN_SECS` (saturating) — a clock set backwards
/// relative to the recorded prompt can only extend, never shorten, the wait.
fn should_request_review(now: u64, last_prompt: Option<u64>, completed_sessions: u64) -> bool {
    if completed_sessions < MIN_SESSIONS_BEFORE_FIRST_PROMPT {
        return false;
    }
    match last_prompt {
        None => true,
        Some(last) => now.saturating_sub(last) >= COOLDOWN_SECS,
    }
}

/// Counts the completed session and, when the gate allows, hands the
/// request to the native review prompt.
///
/// `reviewed_cards` is the session's processed card count (lesson reviews +
/// acquaintance cards) taken from the completion screen. Zero-card sessions
/// are neither counted nor prompted — the complete screen should not even
/// appear for them, so this guard is defense in depth.
///
/// The cooldown timestamp is recorded only when the native side reports the
/// prompt was actually requested (`request_app_review` → `true`); platforms
/// without a wired review API return `false` and never burn the cooldown.
pub fn maybe_request_app_review(reviewed_cards: usize) {
    if reviewed_cards == 0 || !is_tauri() {
        return;
    }

    let now = js_sys::Date::now() as u64 / 1000;
    let completed_sessions: u64 = LocalStorage::get(SESSIONS_KEY).unwrap_or(0) + 1;
    if let Err(e) = LocalStorage::set(SESSIONS_KEY, completed_sessions) {
        tracing::warn!("review_request: failed to persist session count: {e}");
    }

    let last_prompt: Option<u64> = LocalStorage::get(LAST_PROMPT_KEY).ok();
    if !should_request_review(now, last_prompt, completed_sessions) {
        return;
    }

    spawn_local(async move {
        match invoke_with_args("request_app_review", &JsValue::UNDEFINED).await {
            Ok(result) if result.as_bool() == Some(true) => {
                if let Err(e) = LocalStorage::set(LAST_PROMPT_KEY, now) {
                    tracing::warn!("review_request: failed to persist prompt time: {e}");
                }
                tracing::info!("review_request: native review prompt requested");
            },
            Ok(_) => tracing::debug!("review_request: platform has no in-app review API"),
            Err(e) => tracing::warn!("review_request: invoke failed: {e}"),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::should_request_review;
    use rstest::rstest;

    const DAY: u64 = 24 * 60 * 60;
    const NOW: u64 = 1_800_000_000;

    #[rstest]
    #[case::first_prompt_at_minimum_sessions(None, 3, true)]
    #[case::one_session_below_minimum(None, 2, false)]
    #[case::still_in_cooldown(Some(NOW - 119 * DAY), 5, false)]
    #[case::cooldown_boundary_reached(Some(NOW - 120 * DAY), 5, true)]
    #[case::cooldown_elapsed(Some(NOW - 121 * DAY), 5, true)]
    #[case::clock_behind_recorded_prompt(Some(NOW + DAY), 5, false)]
    fn gate_truth_table(
        #[case] last_prompt: Option<u64>,
        #[case] sessions: u64,
        #[case] due: bool,
    ) {
        assert_eq!(should_request_review(NOW, last_prompt, sessions), due);
    }
}
