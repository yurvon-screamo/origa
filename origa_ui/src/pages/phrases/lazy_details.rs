//! Lazy phrase-detail loading helpers of the phrases page (#540-В1):
//! id collection over cards, cache-miss detection and the shared
//! "load a batch + bump the refresh trigger" step.

use leptos::prelude::*;
use origa::dictionary::phrase::get_cached_phrase_detail;
use origa::domain::{Card, StudyCard};
use ulid::Ulid;

use crate::loaders::phrase_data_loader::load_phrase_details_batch;

pub(super) fn phrase_ids_of(cards: &[StudyCard]) -> Vec<Ulid> {
    cards
        .iter()
        .filter_map(|card| {
            if let Card::Phrase(pc) = card.card() {
                Some(*pc.phrase_id())
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn missing_details(ids: &[Ulid]) -> Vec<Ulid> {
    ids.iter()
        .filter(|id| get_cached_phrase_detail(id).is_none())
        .copied()
        .collect()
}

/// Runs one batch load and bumps the refresh trigger when it lands (the
/// cards re-render with the freshly cached details).
pub(super) async fn load_and_refresh(ids: Vec<Ulid>, refresh: RwSignal<u32>) {
    let total = ids.len();
    let results = load_phrase_details_batch(&ids).await;
    let failed = results.iter().filter(|r| r.is_err()).count();
    if failed > 0 {
        tracing::warn!(failed, total, "Some phrase data chunks failed to load");
    }
    refresh.update(|n| *n += 1);
}
