//! Cleanup of lesson phrase cards whose CDN details failed to load.
//!
//! Extracted from `LessonContent` so the permanent-vs-transient retention
//! policy (see #540) is unit-testable without a Leptos reactive context.

use std::collections::HashSet;

use origa::domain::{Card, LessonCard};
use ulid::Ulid;

/// Collects phrase ids whose batch load returned an error. `results` must be
/// aligned with `phrase_ids` element-wise (the contract of
/// `load_phrase_details_batch`).
pub(crate) fn failed_phrase_ids<T, E>(phrase_ids: &[Ulid], results: &[Result<T, E>]) -> Vec<Ulid> {
    phrase_ids
        .iter()
        .zip(results.iter())
        .filter_map(|(id, result)| result.as_ref().err().map(|_| *id))
        .collect()
}

/// Drops phrase cards whose details are PERMANENTLY lost from the lesson,
/// keeping transient failures (chunk not loaded yet — e.g. another batch
/// holds it in flight, #540) so rendering can fall back until the details
/// land. Returns the slot ids of removed cards.
///
/// Invariant: `permanent` must be a subset of the failed phrase ids that
/// were classified by `classify_orphaned_phrases` (this is guaranteed at the
/// call site). Passing ids that were never failed would also remove them —
/// the function intentionally trusts the classifier's verdict.
pub(crate) fn retain_loaded_phrases(
    cards: &mut Vec<(Ulid, LessonCard)>,
    permanent: &HashSet<Ulid>,
) -> Vec<Ulid> {
    let mut removed_card_ids: Vec<Ulid> = Vec::new();
    cards.retain(|(card_id, lesson_card)| {
        if let Card::Phrase(phrase_card) = lesson_card.view().card() {
            if permanent.contains(phrase_card.phrase_id()) {
                removed_card_ids.push(*card_id);
                return false;
            }
        }
        true
    });
    removed_card_ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::{LessonCardView, PhraseCard};

    fn phrase_slot(phrase_id: Ulid) -> (Ulid, LessonCard) {
        let card_id = Ulid::new();
        (
            card_id,
            LessonCard::new(
                card_id,
                LessonCardView::Normal(Card::Phrase(PhraseCard::new(phrase_id))),
                false,
            ),
        )
    }

    #[test]
    fn failed_phrase_ids_picks_only_errored_positions() {
        // Arrange
        let a = Ulid::new();
        let b = Ulid::new();
        let c = Ulid::new();
        let results = vec![Ok::<(), String>(()), Err("boom".to_string()), Ok(())];

        // Act
        let failed = failed_phrase_ids(&[a, b, c], &results);

        // Assert
        assert_eq!(failed, vec![b]);
    }

    #[test]
    fn failed_phrase_ids_empty_when_all_loaded() {
        // Arrange
        let a = Ulid::new();
        let results = vec![Ok::<(), String>(())];

        // Act
        let failed = failed_phrase_ids(&[a], &results);

        // Assert
        assert!(failed.is_empty());
    }

    #[test]
    fn retain_loaded_phrases_drops_permanent_keeps_transient() {
        // Arrange
        let permanent_id = Ulid::new();
        let transient_id = Ulid::new();
        let permanent = phrase_slot(permanent_id);
        let transient = phrase_slot(transient_id);
        let mut cards = vec![permanent.clone(), transient.clone()];
        let mut permanent_set = HashSet::new();
        permanent_set.insert(permanent_id);

        // Act
        let removed = retain_loaded_phrases(&mut cards, &permanent_set);

        // Assert
        assert_eq!(removed, vec![permanent.0]);
        let remaining_phrase_ids: HashSet<Ulid> = cards
            .iter()
            .filter_map(|(_, lc)| match lc.view().card() {
                Card::Phrase(pc) => Some(*pc.phrase_id()),
                _ => None,
            })
            .collect();
        assert_eq!(remaining_phrase_ids, HashSet::from([transient_id]));
    }

    #[test]
    fn retain_loaded_phrases_noop_on_empty_permanent_set() {
        // Arrange
        let mut cards = vec![phrase_slot(Ulid::new()), phrase_slot(Ulid::new())];

        // Act
        let removed = retain_loaded_phrases(&mut cards, &HashSet::new());

        // Assert
        assert!(removed.is_empty());
        assert_eq!(cards.len(), 2);
    }
}
