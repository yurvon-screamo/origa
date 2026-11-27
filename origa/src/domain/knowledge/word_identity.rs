//! Canonical word identity for study cards: the SudachiDict lemma.
//!
//! Card content created through `from_text` already carries this key. This
//! module repairs historical cards and cards arriving through sync; it does
//! not introduce a second notion of word identity.

use std::cmp::Reverse;
use std::collections::HashMap;

use ulid::Ulid;

use crate::domain::knowledge::ExampleCard;
use crate::domain::knowledge::card::ImportDedupKey;
use crate::domain::tokenizer::tokenize_text;
use crate::domain::{Card, NativeLanguage, OrigaError, StudyCard, VocabularyCard};

use super::KnowledgeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WordIdentityCleanupReport {
    pub groups_scanned: usize,
    pub cards_rebased: usize,
    pub cards_removed: usize,
    pub kept: Vec<(String, Ulid)>,
    pub removed: Vec<(Ulid, String)>,
}

pub(super) fn merge_incoming(set: &mut KnowledgeSet, id: Ulid, incoming: &StudyCard) {
    if set.deleted_cards.contains(&id) {
        return;
    }
    if merge_same_id(set, id, incoming) {
        return;
    }
    if let Some(collision) = collision_id(set, incoming.card()) {
        resolve_sync_collision(set, collision, incoming);
        return;
    }
    set.study_cards.insert(id, incoming.clone());
}

fn merge_same_id(set: &mut KnowledgeSet, id: Ulid, incoming: &StudyCard) -> bool {
    let Some(existing) = set.study_cards.get_mut(&id) else {
        return false;
    };
    existing.merge(incoming);
    if canonical_replaces(existing.card(), incoming.card()) {
        existing.replace_card(incoming.card().clone());
    }
    true
}

pub(super) fn collision_id(set: &KnowledgeSet, card: &Card) -> Option<Ulid> {
    let key = ImportDedupKey::new(card);
    set.study_cards()
        .iter()
        .find_map(|(id, existing)| (ImportDedupKey::new(existing.card()) == key).then_some(*id))
}

pub(super) fn example_count(set: &KnowledgeSet, word: &str) -> usize {
    set.study_cards()
        .values()
        .filter(|study| matches!(study.card(), Card::Example(example) if example.word() == word))
        .count()
}

pub(super) fn resolve_sync_collision(set: &mut KnowledgeSet, local_id: Ulid, incoming: &StudyCard) {
    let key_word = match incoming.card() {
        Card::Example(example) => example.word().to_string(),
        _ => incoming.card().content_key(),
    };
    let examples = example_count(set, &key_word);
    let local_wins = set
        .study_cards()
        .get(&local_id)
        .is_some_and(|local| priority(local, examples) >= priority(incoming, examples));
    if local_wins {
        if let Some(local) = set.study_cards.get_mut(&local_id) {
            local.merge(incoming);
        }
        return;
    }

    let Some(local) = set.study_cards.remove(&local_id) else {
        return;
    };
    let mut winner = incoming.clone();
    winner.merge(&local);
    set.deleted_cards.insert(local_id);
    set.study_cards.insert(*incoming.card_id(), winner);
}

pub fn merge_canonicalized(
    set: &mut KnowledgeSet,
    new_values: &KnowledgeSet,
    lang: &NativeLanguage,
) {
    let canonical = canonical_snapshot(&new_values.study_cards, lang);
    set.merge(&KnowledgeSet {
        study_cards: canonical,
        ..new_values.clone()
    });
}

pub fn cleanup(
    set: &mut KnowledgeSet,
    lang: &NativeLanguage,
) -> Result<WordIdentityCleanupReport, OrigaError> {
    let mut report = WordIdentityCleanupReport::default();
    let ids: Vec<Ulid> = set.study_cards().keys().copied().collect();

    for id in ids {
        let Some(study) = set.study_cards().get(&id).cloned() else {
            continue;
        };
        let canonical = canonicalize_card(study.card(), lang);
        if &canonical != study.card() {
            report.cards_rebased += 1;
            if let Some(card) = set.study_cards.get_mut(&id) {
                card.replace_card(canonical);
            }
        }
    }

    let mut groups: HashMap<ImportDedupKey, Vec<Ulid>> = HashMap::new();
    for (id, study) in set.study_cards() {
        groups
            .entry(ImportDedupKey::new(study.card()))
            .or_default()
            .push(*id);
    }

    for (_key, mut ids) in groups {
        if ids.len() < 2 {
            continue;
        }
        report.groups_scanned += 1;
        ids.sort_by_key(|id| {
            set.study_cards()
                .get(id)
                .map(|card| {
                    let word = match card.card() {
                        Card::Example(example) => example.word().to_string(),
                        _ => card.card().content_key(),
                    };
                    priority(card, example_count(set, &word))
                })
                .unwrap_or_default()
        });
        ids.reverse();
        let winner_id = ids[0];
        let mut winner = set
            .study_cards()
            .get(&winner_id)
            .cloned()
            .ok_or(OrigaError::CardNotFound { card_id: winner_id })?;
        let content_key = winner.card().content_key();
        report.kept.push((content_key.clone(), winner_id));
        for loser_id in &ids[1..] {
            let loser = set
                .study_cards()
                .get(loser_id)
                .cloned()
                .ok_or(OrigaError::CardNotFound { card_id: *loser_id })?;
            winner.merge(&loser);
            set.delete_card(*loser_id)?;
            report.cards_removed += 1;
            report.removed.push((*loser_id, content_key.clone()));
        }
        set.study_cards.insert(winner_id, winner);
    }

    Ok(report)
}

pub(super) fn lemma_of(word: &str) -> String {
    tokenize_text(word)
        .ok()
        .and_then(|tokens| {
            tokens
                .first()
                .map(|token| token.orthographic_base_form().to_string())
        })
        .unwrap_or_else(|| word.to_string())
}

pub(super) fn is_canonical(word: &str) -> bool {
    lemma_of(word) == word
}

pub(super) fn canonical_replaces(local: &Card, incoming: &Card) -> bool {
    match (local, incoming) {
        (Card::Vocabulary(local), Card::Vocabulary(incoming)) => {
            !is_canonical(local.word().text()) && is_canonical(incoming.word().text())
        },
        (Card::Example(local), Card::Example(incoming)) => {
            !is_canonical(local.word()) && is_canonical(incoming.word())
        },
        _ => false,
    }
}

pub(super) fn canonicalize_card(card: &Card, lang: &NativeLanguage) -> Card {
    match card {
        Card::Vocabulary(vocabulary) => {
            let lemma = lemma_of(vocabulary.word().text());
            if lemma == vocabulary.word().text() {
                return card.clone();
            }
            VocabularyCard::from_known_word(&lemma, lang)
                .map(Card::Vocabulary)
                .unwrap_or_else(|_| card.clone())
        },
        Card::Example(example) => {
            let lemma = lemma_of(example.word());
            if lemma == example.word() {
                return card.clone();
            }
            Card::Example(ExampleCard::new(lemma, example.sentence_id()))
        },
        _ => card.clone(),
    }
}

pub(super) fn canonical_snapshot(
    cards: &HashMap<Ulid, StudyCard>,
    lang: &NativeLanguage,
) -> HashMap<Ulid, StudyCard> {
    cards
        .iter()
        .map(|(id, card)| {
            let mut canonical = card.clone();
            canonical.replace_card(canonicalize_card(card.card(), lang));
            (*id, canonical)
        })
        .collect()
}

/// Winner priority shared by local cleanup and the sync boundary. `Reverse`
/// on the id makes the smaller Ulid win the final tie-break.
pub(super) type Priority = (u64, u32, bool, bool, usize, Reverse<Ulid>);

pub(super) fn priority(card: &StudyCard, example_count: usize) -> Priority {
    let stability = card
        .memory()
        .stability()
        .map(|value| (value.value() * 1_000.0) as u64)
        .unwrap_or_default();
    (
        stability,
        card.memory().reps(),
        card.memory().is_known_card(),
        card.is_favorite(),
        example_count,
        Reverse(*card.card_id()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::real_dictionaries_or_skip;
    use crate::domain::knowledge::{KnowledgeSet, VocabularyCard};
    use crate::domain::value_objects::Question;

    fn surface_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    #[test]
    fn lemma_of_maps_script_and_inflection_to_one_form() {
        real_dictionaries_or_skip!();
        assert_eq!(lemma_of("たべます"), "食べる");
        assert_eq!(lemma_of("たべる"), "食べる");
    }

    #[test]
    fn lemma_of_keeps_independent_article() {
        real_dictionaries_or_skip!();
        assert_eq!(lemma_of("見える"), "見える");
    }

    #[test]
    fn merge_replaces_dirty_local_card_when_same_id_arrives_canonical() {
        real_dictionaries_or_skip!();
        // Arrange
        let mut local = KnowledgeSet::new();
        let study = local.create_card(surface_card("たべる")).unwrap();
        let id = *study.card_id();
        let mut incoming = study.clone();
        incoming.replace_card(surface_card("食べる"));
        let mut remote = KnowledgeSet::new();
        remote.study_cards.insert(id, incoming);

        // Act
        local.merge(&remote);

        // Assert
        let merged = local.study_cards.get(&id).unwrap();
        assert_eq!(merged.card().content_key(), "食べる");
    }

    #[test]
    fn merge_keeps_smaller_ulid_for_duplicate_words_with_different_ids() {
        real_dictionaries_or_skip!();
        // Arrange
        let mut local = KnowledgeSet::new();
        local.create_card(surface_card("食べる")).unwrap();
        let local_id = *local.study_cards.keys().next().unwrap();
        let mut remote = KnowledgeSet::new();
        remote.create_card(surface_card("食べる")).unwrap();
        let remote_id = *remote.study_cards.keys().next().unwrap();
        let winner = if local_id < remote_id {
            local_id
        } else {
            remote_id
        };

        // Act
        local.merge(&remote);

        // Assert
        assert_eq!(local.study_cards.len(), 1);
        assert!(local.study_cards.contains_key(&winner));
    }

    #[test]
    fn merge_keeps_one_example_card_per_lemma_and_sentence() {
        real_dictionaries_or_skip!();
        // Arrange
        let mut local = KnowledgeSet::new();
        local
            .create_card(Card::Example(ExampleCard::new("食べる", 42)))
            .unwrap();
        let mut remote = KnowledgeSet::new();
        remote
            .create_card(Card::Example(ExampleCard::new("食べる", 42)))
            .unwrap();

        // Act
        local.merge(&remote);

        // Assert
        assert_eq!(local.study_cards.len(), 1);
        assert!(matches!(
            local.study_cards.values().next().unwrap().card(),
            Card::Example(example) if example.word() == "食べる" && example.sentence_id() == 42
        ));
    }
}
