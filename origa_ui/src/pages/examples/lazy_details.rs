//! Lazy example-detail loading helpers of the examples page (#823):
//! sentence-id collection over cards, cache-miss detection and the shared
//! "load a batch + bump the refresh trigger" step. Mirrors the phrases
//! page (#540-В1), keyed by sentence id instead of Ulid.

use leptos::prelude::*;
use origa::dictionary::example::get_cached_example_detail;
use origa::domain::{Card, StudyCard};

use crate::loaders::example_loader::load_example_detail;

pub(super) fn sentence_ids_of(cards: &[StudyCard]) -> Vec<u32> {
    cards
        .iter()
        .filter_map(|card| {
            if let Card::Example(ec) = card.card() {
                Some(ec.sentence_id())
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn missing_details(sentence_ids: &[u32]) -> Vec<u32> {
    sentence_ids
        .iter()
        .filter(|id| get_cached_example_detail(**id).is_none())
        .copied()
        .collect()
}

/// Runs one batch load and bumps the refresh trigger when it lands (the
/// cards re-render with the freshly cached details).
pub(super) async fn load_and_refresh(sentence_ids: Vec<u32>, refresh: RwSignal<u32>) {
    let total = sentence_ids.len();
    let results =
        futures::future::join_all(sentence_ids.iter().map(|id| load_example_detail(*id))).await;
    let failed = results.iter().filter(|r| r.is_err()).count();
    if failed > 0 {
        tracing::warn!(failed, total, "Some example data chunks failed to load");
    }
    refresh.update(|n| *n += 1);
}
