//! Scroll save/restore for the card list pages.
//!
//! The save side tracks the window `scroll` event and records the position
//! as the user scrolls — never on unmount. Reading the position in
//! `on_cleanup` is unreliable: by the time the old route's DOM is detached
//! the browser has already clamped the scroll to the next document, so the
//! cleanup would record a clamped (often zero) value. The restore side runs
//! once per mount, after the rendered content is ready. Both sides share
//! [`ListUiSlot::scroll_y`], which the store keeps for the whole session —
//! that is what makes "come back where you left off" work.

use super::ListUiSlot;
use crate::store::auth_store::AuthStore;
use crate::utils::yield_to_browser;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::use_event_listener;

/// Starts recording the window scroll position into `slot`.
///
/// The auth guard keeps the tracker from writing after the session ended
/// (logout clears the store while the list page is still tearing down).
/// A missing `AuthStore` context degrades to "do not save". The listener
/// detaches itself with the page's owner — no manual cleanup needed.
pub fn track_scroll_while_authenticated(slot: &ListUiSlot) {
    let auth = use_context::<AuthStore>();
    let authenticated = auth.map(|auth| auth.is_authenticated());
    let slot = slot.clone();
    let _ = use_event_listener(window(), leptos::ev::scroll, move |_| {
        let still_signed_in = authenticated.as_ref().is_some_and(|s| s.get_untracked());
        if !still_signed_in {
            return;
        }
        if let Some(window) = web_sys::window() {
            slot.scroll_y.set_value(window.scroll_y().unwrap_or(0.0));
        }
    });
}

/// Restores the saved scroll position once per mount, when the list has
/// content on screen (`is_loading` cleared and `content_ready` true).
///
/// Guards, in order: the restore runs at most once per mount (later effect
/// reruns — e.g. "load more" — must be no-ops), and a non-positive saved
/// position is skipped entirely. The restore is scheduled through two
/// browser yields so the rendered list has been laid out and painted by
/// the time the position is applied; the browser clamps to the document
/// height if the content got shorter — the safe degradation is staying at
/// the top.
pub fn restore_scroll_when_ready(
    slot: &ListUiSlot,
    is_loading: RwSignal<bool>,
    content_ready: impl Fn() -> bool + Send + Sync + 'static,
) {
    let slot = slot.clone();
    let done = StoredValue::new(false);
    Effect::new(move |_| {
        if done.get_value() || is_loading.get() || !content_ready() {
            return;
        }
        done.set_value(true);
        let saved_y = slot.scroll_y.get_value();
        if saved_y <= 0.0 {
            return;
        }
        spawn_local(async move {
            yield_to_browser().await;
            yield_to_browser().await;
            if let Some(window) = web_sys::window() {
                window.scroll_to_with_x_and_y(0.0, saved_y);
                tracing::debug!(saved_y, "Restored list scroll position");
            }
        });
    });
}
