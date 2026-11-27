//! Read-only scan of duplicate cards by their stored content key.
//!
//! Word creation resolves the SudachiDict lemma once in `from_text`. Re-tokenizing
//! an already stored word is NOT equivalent (normalized forms are not
//! idempotent), so this module never rewrites card content — it only reports
//! groups that share the exact key the creation paths use.

use std::collections::HashMap;

use ulid::Ulid;

use crate::domain::KnowledgeSet;
use crate::domain::knowledge::card::ImportDedupKey;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WordIdentityScanReport {
    pub groups_scanned: usize,
    pub duplicate_cards: usize,
    pub groups: Vec<(String, Vec<Ulid>)>,
}

/// Groups cards by the exact `(CardType, content_key)` identity used by
/// `validate_unique_card`. No card is mutated or deleted.
pub fn scan_duplicate_cards(set: &KnowledgeSet) -> WordIdentityScanReport {
    let mut groups: HashMap<ImportDedupKey, Vec<Ulid>> = HashMap::new();
    for (id, study) in set.study_cards() {
        groups
            .entry(ImportDedupKey::new(study.card()))
            .or_default()
            .push(*id);
    }

    let mut report = WordIdentityScanReport::default();
    for (_key, mut ids) in groups {
        if ids.len() < 2 {
            continue;
        }
        ids.sort_unstable();
        report.groups_scanned += 1;
        report.duplicate_cards += ids.len() - 1;
        report.groups.push((key_to_label(set, &ids[0]), ids));
    }
    report.groups.sort_by(|left, right| left.0.cmp(&right.0));
    report
}

fn key_to_label(set: &KnowledgeSet, id: &Ulid) -> String {
    set.study_cards()
        .get(id)
        .map(|study| study.card().content_key())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::knowledge::VocabularyCard;
    use crate::domain::value_objects::Question;
    use crate::domain::{Card, StudyCard};

    fn card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    #[test]
    fn scan_reports_exact_duplicates_without_mutating_deck() {
        // Arrange
        let mut set = KnowledgeSet::new();
        set.create_card(card("食べる")).unwrap();
        set.study_cards
            .insert(ulid::Ulid::new(), StudyCard::new(card("食べる")));
        set.create_card(card("見える")).unwrap();
        let before = set.study_cards().len();

        // Act
        let report = scan_duplicate_cards(&set);

        // Assert
        assert_eq!(report.groups_scanned, 1);
        assert_eq!(report.duplicate_cards, 1);
        assert_eq!(report.groups[0].0, "食べる");
        assert_eq!(set.study_cards().len(), before);
    }

    #[test]
    fn scan_keeps_distinct_articles_in_separate_groups() {
        // Arrange
        let mut set = KnowledgeSet::new();
        set.create_card(card("止める")).unwrap();
        set.create_card(card("止む")).unwrap();

        // Act
        let report = scan_duplicate_cards(&set);

        // Assert
        assert_eq!(report.groups_scanned, 0);
        assert_eq!(report.duplicate_cards, 0);
    }
}
