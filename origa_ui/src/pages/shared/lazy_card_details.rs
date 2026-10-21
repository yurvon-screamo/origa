//! Shared lazy-detail loading machinery of the card list pages
//! (#540-В1, #823): the visible-slice load loop and the search-driven
//! backfill, parameterized by the page's detail-id type (Ulid for
//! phrases, u32 sentence ids for examples) and its three detail hooks.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use futures::future::LocalBoxFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::StudyCard;

use super::card_list_page::VisibleCardsCallback;

/// Page-side detail hooks of the lazy loaders.
///
/// - `ids_of` collects the detail ids carried by a card slice.
/// - `missing_of` filters out the ids whose details are already cached.
/// - `load_batch` loads one batch of details and bumps the refresh
///   trigger when it lands (the cards re-render from the cache).
#[derive(Clone)]
pub struct LazyDetailHooks<T> {
    pub ids_of: IdsOf<T>,
    pub missing_of: MissingOf<T>,
    pub load_batch: LoadBatch<T>,
}

pub type IdsOf<T> = Arc<dyn Fn(&[StudyCard]) -> Vec<T> + Send + Sync>;
pub type MissingOf<T> = Arc<dyn Fn(&[T]) -> Vec<T> + Send + Sync>;
pub type LoadBatch<T> = Arc<dyn Fn(Vec<T>) -> LocalBoxFuture<'static, ()> + Send + Sync>;

/// Builds the visible-slice lazy-load trigger: runs on first render,
/// "load more" and filter/search switches. While a load is in flight,
/// later triggers only mark a rerun — the loop picks the freshest
/// visible ids after the current batch lands (no lost tail, no
/// duplicate batches).
///
/// The loop is unscoped with a disposed guard (NOT scoped): the ambient
/// owner at the spawn point is whatever context the list component
/// invokes this callback from — unpredictable for scoping. The loop-top
/// reads `visible_ids` on every rerun pass (i.e. after an await); the
/// sentinel fence stops the task at the top of the next pass once the
/// page is disposed (disposed-signal fix). A snapshot would defeat the
/// rerun flag — each pass must read the fresh ids.
pub fn create_visible_slice_loader<T>(
    backfill_running: RwSignal<bool>,
    hooks: LazyDetailHooks<T>,
) -> VisibleCardsCallback
where
    T: Copy + Send + Sync + 'static,
{
    let visible_ids: RwSignal<Vec<T>> = RwSignal::new(Vec::new());
    let running = Arc::new(AtomicBool::new(false));
    let rerun = Arc::new(AtomicBool::new(false));
    let disposed = StoredValue::new(());
    let LazyDetailHooks {
        ids_of,
        missing_of,
        load_batch,
    } = hooks;

    Arc::new(move |cards: &[StudyCard]| {
        // The search backfill covers EVERY card — a visible-slice load
        // racing it would only collect Errs on the in-flight chunks.
        // Skip; the backfill's refresh re-renders the cards.
        if backfill_running.get_untracked() {
            return;
        }
        let ids = ids_of(cards);
        if missing_of(&ids).is_empty() {
            return;
        }
        visible_ids.set(ids);
        if running.swap(true, Ordering::AcqRel) {
            rerun.store(true, Ordering::Release);
            return;
        }
        let running = running.clone();
        let rerun = rerun.clone();
        let load_batch = load_batch.clone();
        spawn_local(async move {
            loop {
                if disposed.is_disposed() {
                    return;
                }
                let ids = visible_ids.get_untracked();
                if !ids.is_empty() {
                    (load_batch)(ids).await;
                }
                if !rerun.swap(false, Ordering::AcqRel) {
                    break;
                }
            }
            running.store(false, Ordering::Release);
        });
    })
}

/// Session state of the search-driven backfill: the observed query, the
/// full id list of the session and the done/loading flags.
#[derive(Clone, Copy)]
pub struct BackfillState<T> {
    pub search: RwSignal<String>,
    pub all_ids: RwSignal<Vec<T>>,
    pub done: RwSignal<bool>,
    pub loading: RwSignal<bool>,
}

/// Installs the search-driven backfill effect: the first non-empty query
/// of a session loads the details of EVERY card once, so search matches
/// against the complete set. Matching over already-loaded details is
/// instant — results keep appearing as the batch lands (the browser
/// serializes the fetches over its connection pool anyway).
///
/// Takes only the hooks it uses (`ids_of` is the visible-slice loader's
/// concern) — the page moves them out of its `LazyDetailHooks` value.
pub fn create_search_backfill_effect<T>(
    state: BackfillState<T>,
    label: &'static str,
    missing_of: MissingOf<T>,
    load_batch: LoadBatch<T>,
) where
    T: Copy + Send + Sync + 'static,
{
    let BackfillState {
        search,
        all_ids,
        done,
        loading,
    } = state;
    Effect::new(move |_| {
        let query = search.get();
        if query.trim().is_empty() || done.get_untracked() || loading.get_untracked() {
            return;
        }
        let missing = missing_of(&all_ids.get_untracked());
        if missing.is_empty() {
            done.set(true);
            return;
        }
        loading.set(true);
        let load_batch = load_batch.clone();
        spawn_local(async move {
            tracing::info!(
                missing = missing.len(),
                label,
                "Backfilling card details for search"
            );
            (load_batch)(missing).await;
            loading.set(false);
            done.set(true);
        });
    });
}
