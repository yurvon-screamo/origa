//! Native tests for [`super::list_ui_state`]. Kept in a separate file to
//! keep the store module itself under the project's size limit.

#[cfg(test)]
mod tests {
    use crate::pages::shared::list_ui_state::*;
    use crate::pages::shared::{Filter, JlptFilter};
    use leptos::prelude::*;
    use origa::domain::JapaneseLevel;

    fn owner_with<T>(f: impl FnOnce() -> T) -> T {
        Owner::new().with(f)
    }

    #[test]
    fn slots_start_at_defaults() {
        owner_with(|| {
            let store = ListUiStore::new();
            let slot = store.slot(ListPage::Phrases);
            assert_eq!(slot.search.get(), "");
            assert_eq!(slot.status.get(), Filter::All);
            assert_eq!(slot.jlpt.get(), JlptFilter::All);
            assert_eq!(slot.visible_count.get(), DEFAULT_VISIBLE_COUNT);
            assert_eq!(slot.scroll_y.get_value(), 0.0);
        });
    }

    #[test]
    fn slot_returns_the_same_signals_for_the_same_page() {
        owner_with(|| {
            let store = ListUiStore::new();
            let first = store.slot(ListPage::Grammar);
            let second = store.slot(ListPage::Grammar);
            assert_eq!(first.search, second.search);
            assert_eq!(first.status, second.status);
            assert_eq!(first.jlpt, second.jlpt);
            assert_eq!(first.visible_count, second.visible_count);
        });
    }

    #[test]
    fn slots_of_different_pages_are_independent() {
        owner_with(|| {
            let store = ListUiStore::new();
            let words = store.slot(ListPage::Words);
            let kanji = store.slot(ListPage::Kanji);
            assert_ne!(words.search, kanji.search);
            assert_ne!(words.status, kanji.status);
            assert_ne!(words.jlpt, kanji.jlpt);
            assert_ne!(words.visible_count, kanji.visible_count);

            // Counters — отдельная страница (issue #415): слот независим.
            let counters = store.slot(ListPage::Counters);
            assert_ne!(kanji.search, counters.search);
            assert_ne!(kanji.jlpt, counters.jlpt);
            assert_ne!(kanji.visible_count, counters.visible_count);
        });
    }

    #[test]
    fn reset_returns_every_page_to_defaults() {
        owner_with(|| {
            let store = ListUiStore::new();
            let words = store.slot(ListPage::Words);
            let grammar = store.slot(ListPage::Grammar);
            words.search.set("food".to_string());
            words.status.set(Filter::Hard);
            words.visible_count.set(150);
            words.scroll_y.set_value(420.0);
            grammar.jlpt.set(JlptFilter::Level(JapaneseLevel::N3));

            store.reset();

            assert_eq!(words.search.get(), "");
            assert_eq!(words.status.get(), Filter::All);
            assert_eq!(words.visible_count.get(), DEFAULT_VISIBLE_COUNT);
            assert_eq!(words.scroll_y.get_value(), 0.0);
            assert_eq!(grammar.jlpt.get(), JlptFilter::All);
        });
    }

    #[test]
    fn first_run_guard_skips_only_the_first_call() {
        owner_with(|| {
            let guard = FirstRunGuard::new();
            assert!(!guard.should_run());
            assert!(guard.should_run());
            assert!(guard.should_run());
        });
    }
}
