//! Session-scoped UI state of the card list pages (`/words`, `/kanji`,
//! `/grammar`, `/phrases`).
//!
//! Filter, search and pagination signals live here instead of inside
//! [`super::card_list_view`], so navigating to a detail page and back keeps
//! the list exactly as the user left it. The store is provided as app-level
//! context in `app.rs` and lives as long as the session; a logout resets it
//! so state never crosses users.

use super::{Filter, JlptFilter};
use leptos::prelude::*;

/// Initial value of the rendered card slice, restored to it whenever the
/// active filter/search changes (`card_list_view` reset effect).
pub const DEFAULT_VISIBLE_COUNT: usize = 50;

/// Identifies a list page so the store can hand out its state slot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ListPage {
    Words,
    Kanji,
    Grammar,
    Phrases,
}

/// Reactive state of one list page. Every field is `Copy` and shares the
/// underlying signal — cloning the slot does not fork the state.
#[derive(Clone)]
pub struct ListUiSlot {
    pub search: RwSignal<String>,
    pub status: RwSignal<Filter>,
    pub jlpt: RwSignal<JlptFilter>,
    pub visible_count: RwSignal<usize>,
    /// Non-reactive on purpose: written by the window scroll tracker as the
    /// user scrolls (`list_scroll.rs`), read once on mount — a reactive
    /// client would only create spurious subscriptions.
    pub scroll_y: StoredValue<f64>,
}

/// Session-scoped holder of every list page's UI state.
#[derive(Clone)]
pub struct ListUiStore {
    words: ListUiSlot,
    kanji: ListUiSlot,
    grammar: ListUiSlot,
    phrases: ListUiSlot,
}

impl Default for ListUiStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ListUiStore {
    /// Must be called inside a reactive owner (app bootstrap or a test
    /// owner scope) — the slots create their signals here.
    pub fn new() -> Self {
        Self {
            words: Self::make_slot(),
            kanji: Self::make_slot(),
            grammar: Self::make_slot(),
            phrases: Self::make_slot(),
        }
    }

    fn make_slot() -> ListUiSlot {
        ListUiSlot {
            search: RwSignal::new(String::new()),
            status: RwSignal::new(Filter::All),
            jlpt: RwSignal::new(JlptFilter::All),
            visible_count: RwSignal::new(DEFAULT_VISIBLE_COUNT),
            scroll_y: StoredValue::new(0.0),
        }
    }

    /// Returns the state slot of `page`. Idempotent: every call for the
    /// same page returns the same signals for the whole session.
    pub fn slot(&self, page: ListPage) -> ListUiSlot {
        match page {
            ListPage::Words => self.words.clone(),
            ListPage::Kanji => self.kanji.clone(),
            ListPage::Grammar => self.grammar.clone(),
            ListPage::Phrases => self.phrases.clone(),
        }
    }

    /// Returns every list page to its defaults. Called when the session
    /// ends (logout) so filters and scroll never cross users.
    pub fn reset(&self) {
        for slot in [&self.words, &self.kanji, &self.grammar, &self.phrases] {
            slot.search.set(String::new());
            slot.status.set(Filter::All);
            slot.jlpt.set(JlptFilter::All);
            slot.visible_count.set(DEFAULT_VISIBLE_COUNT);
            slot.scroll_y.set_value(0.0);
        }
    }
}

/// Skips the first invocation of a reactive effect.
///
/// Leptos effects always run once on creation; the visible-count reset in
/// `card_list_view` must not fire on that first run — it would clobber the
/// value restored from the store right after remounting the page.
#[derive(Clone)]
pub struct FirstRunGuard {
    first: StoredValue<bool>,
}

impl FirstRunGuard {
    pub fn new() -> Self {
        Self {
            first: StoredValue::new(true),
        }
    }

    /// Returns `true` for every call except the first one.
    pub fn should_run(&self) -> bool {
        if self.first.get_value() {
            self.first.set_value(false);
            return false;
        }
        true
    }
}

impl Default for FirstRunGuard {
    fn default() -> Self {
        Self::new()
    }
}
